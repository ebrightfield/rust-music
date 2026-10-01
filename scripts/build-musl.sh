#!/usr/bin/env bash
# Build this workspace's binaries as fully static musl executables, for copy-pasting
# into containers (scratch, alpine, distroless — anything).
#
# Default is a containerized build (reproducible, pinned toolchain, no host setup).
# `--host` uses the host cargo + rustup musl target instead, which is much faster for
# iteration but bakes in the host's toolchain version.
#
#   ./scripts/build-musl.sh                 # containerized (podman, falls back to docker)
#   ./scripts/build-musl.sh --host          # host cargo, needs the musl target installed
#   ./scripts/build-musl.sh --strip         # also strip symbols (9.3M -> 7.9M)
#   ./scripts/build-musl.sh --prove         # also RUN it in scratch/alpine/debian/busybox
#   ./scripts/build-musl.sh --host --strip
#
# Artifacts land in target/musl-build/ (containerized) or
# target/x86_64-unknown-linux-musl/release/ (host).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

MUSL_TARGET="x86_64-unknown-linux-musl"
IMAGE="rust-music-musl:latest"
OUT_DIR="$REPO_ROOT/target/musl-build"
CONTAINERFILE="Containerfile.musl"

MODE="container"
STRIP=0
PROVE=0
for arg in "$@"; do
  case "$arg" in
    --host)      MODE="host" ;;
    --container) MODE="container" ;;
    --strip)     STRIP=1 ;;
    --prove)     PROVE=1 ;;
    -h|--help)   sed -n '2,20p' "$0" | sed 's/^# \?//'; exit 0 ;;
    *) echo "unknown option: $arg (try --help)" >&2; exit 2 ;;
  esac
done

# ---------------------------------------------------------------------------
# Keep the Containerfile's manifest list in sync with the workspace members.
# A member missing from the COPY list silently breaks the dependency cache layer,
# so check it rather than waiting for a confusing cargo error inside the build.
# ---------------------------------------------------------------------------
check_manifest_list() {
  local members missing=()
  members=$(python3 - <<'PY'
import re, sys
src = open("Cargo.toml").read()
m = re.search(r'^members\s*=\s*\[(.*?)\]', src, re.S | re.M)
if not m:
    sys.exit(0)
for name in re.findall(r'"([^"]+)"', m.group(1)):
    print(name)
PY
)
  for member in $members; do
    if ! grep -q "^COPY $member/Cargo.toml" "$CONTAINERFILE"; then
      missing+=("$member")
    fi
  done
  if (( ${#missing[@]} )); then
    echo "ERROR: $CONTAINERFILE is missing manifest COPY lines for: ${missing[*]}" >&2
    echo "       Add 'COPY <member>/Cargo.toml <member>/Cargo.toml' and a 'COPY <member>/ <member>/'." >&2
    exit 1
  fi
}

# Assert a binary is static and actually runs. This is the whole point of the exercise:
# a dynamically-linked artifact looks fine on the host and dies inside a scratch image.
verify() {
  local bin="$1" rc=0
  [[ -x "$bin" ]] || { echo "ERROR: $bin not found or not executable" >&2; return 1; }

  # The authoritative test is whether the ELF requests a program interpreter (INTERP).
  # A truly static binary has none. Do NOT use `file` for this: it reports "dynamically
  # linked" for binaries that `ldd` simultaneously calls "statically linked", and it
  # reported plausible-looking output for artifacts that segfaulted outside the builder.
  if readelf -l "$bin" 2>/dev/null | grep -q 'program interpreter'; then
    echo "  FAIL  $(basename "$bin"): requests a program interpreter (not static)" >&2
    readelf -l "$bin" | grep 'program interpreter' >&2
    return 1
  fi
  if ldd "$bin" 2>&1 | grep -qE '=> */'; then
    echo "  FAIL  $(basename "$bin"): resolves shared libraries" >&2
    ldd "$bin" >&2
    return 1
  fi
  # Structural proof, stronger than running it here: a binary that needs no shared object
  # has no DT_NEEDED entries. A static-PIE still has a dynamic *section* (it self-relocates),
  # so the presence of that section is not a failure — only DT_NEEDED is.
  local needed
  needed=$(readelf -d "$bin" 2>/dev/null | grep -c 'NEEDED' || true)
  if [[ "${needed:-0}" -ne 0 ]]; then
    echo "  FAIL  $(basename "$bin"): has $needed DT_NEEDED shared-library dependencies" >&2
    readelf -d "$bin" | grep 'NEEDED' >&2
    return 1
  fi
  "$bin" --version >/dev/null 2>&1 || rc=$?
  if (( rc != 0 )); then
    echo "  FAIL  $(basename "$bin"): --version exited $rc" >&2
    return 1
  fi
  printf '  OK    %-14s %6s  %s\n' \
    "$(basename "$bin")" \
    "$(du -h "$bin" | cut -f1)" \
    "$("$bin" --version 2>/dev/null | head -1)"
}

check_manifest_list

if [[ "$MODE" == host ]]; then
  if ! rustup target list --installed 2>/dev/null | grep -q "^$MUSL_TARGET$"; then
    echo "ERROR: rust target $MUSL_TARGET is not installed." >&2
    echo "       rustup target add $MUSL_TARGET" >&2
    exit 1
  fi
  echo "=== Host musl build ($MUSL_TARGET) ==="
  # `link-self-contained=yes` matters: on newer toolchains (verified on rustc 1.97.1)
  # `+crt-static` alone still emits an INTERP segment once a proc-macro dependency is in
  # the graph, producing a binary that segfaults outside an image with musl installed.
  RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+crt-static -C link-self-contained=yes" \
    cargo build --release --target "$MUSL_TARGET" -p slonimsky
  BIN_DIR="$REPO_ROOT/target/$MUSL_TARGET/release"
else
  RUNTIME=""
  for candidate in podman docker; do
    if command -v "$candidate" >/dev/null 2>&1; then RUNTIME="$candidate"; break; fi
  done
  if [[ -z "$RUNTIME" ]]; then
    echo "ERROR: neither podman nor docker found. Use --host for a host build." >&2
    exit 1
  fi

  echo "=== Containerized musl build ($RUNTIME) ==="
  "$RUNTIME" build -f "$CONTAINERFILE" -t "$IMAGE" .

  echo ""
  echo "=== Extracting artifacts to target/musl-build/ ==="
  mkdir -p "$OUT_DIR"
  cid="$("$RUNTIME" create "$IMAGE")"
  trap '"$RUNTIME" rm -f "$cid" >/dev/null 2>&1 || true' EXIT
  "$RUNTIME" cp "$cid":/out/. "$OUT_DIR/"
  "$RUNTIME" rm "$cid" >/dev/null
  trap - EXIT
  BIN_DIR="$OUT_DIR"
fi

if (( STRIP )); then
  echo ""
  echo "=== Stripping ==="
  for bin in "$BIN_DIR"/*; do
    [[ -f "$bin" && -x "$bin" ]] || continue
    before=$(du -h "$bin" | cut -f1)
    strip "$bin"
    echo "  $(basename "$bin"): $before -> $(du -h "$bin" | cut -f1)"
  done
fi

echo ""
echo "=== Verifying static linkage ==="
failed=0
for bin in "$BIN_DIR"/*; do
  [[ -f "$bin" && -x "$bin" ]] || continue
  verify "$bin" || failed=1
done
(( failed == 0 )) || { echo "" >&2; echo "Static verification FAILED." >&2; exit 1; }

# ---------------------------------------------------------------------------
# Portability proof (opt-in with --prove): actually run each binary in foreign libc
# environments and exercise a code path that touches the embedded font + rasterizer.
# The ELF checks above are structural; this is behavioural evidence that the artifact
# is portable rather than merely well-formed on this host.
# ---------------------------------------------------------------------------
if (( PROVE )); then
  RUNTIME="${RUNTIME:-}"
  if [[ -z "$RUNTIME" ]]; then
    for candidate in podman docker; do
      command -v "$candidate" >/dev/null 2>&1 && { RUNTIME="$candidate"; break; }
    done
  fi
  if [[ -z "$RUNTIME" ]]; then
    echo "" >&2; echo "--prove needs podman or docker." >&2; exit 1
  fi

  echo ""
  echo "=== Portability proof ($RUNTIME) ==="
  probe_dir="$(mktemp -d)"
  trap 'rm -rf "$probe_dir"' EXIT
  cp "$BIN_DIR"/* "$probe_dir/" 2>/dev/null || true

  # scratch has no shell, so the binary is the entrypoint; the others get a bind mount.
  printf 'FROM scratch\nCOPY slonimsky /slonimsky\n' > "$probe_dir/Containerfile"
  "$RUNTIME" build -q -t rust-music-musl-probe:latest "$probe_dir" >/dev/null

  pfail=0
  out=$("$RUNTIME" run --rm rust-music-musl-probe:latest /slonimsky --version 2>&1) \
    && printf '  OK    %-22s %s\n' "scratch (empty image)" "$out" \
    || { echo "  FAIL  scratch: $out" >&2; pfail=1; }

  for img in docker.io/library/alpine:3.20 docker.io/library/debian:bookworm-slim docker.io/library/busybox:latest; do
    label="${img##*/}"
    out=$("$RUNTIME" run --rm -v "$probe_dir/slonimsky:/s:z" "$img" /s --version 2>&1 | tail -1) \
      && printf '  OK    %-22s %s\n' "$label" "$out" \
      || { echo "  FAIL  $label: $out" >&2; pfail=1; }
  done

  # Exercise the embedded Bravura font + resvg rasterizer inside the empty image.
  if "$RUNTIME" run --rm -v "$probe_dir:/w:z" rust-music-musl-probe:latest \
       /slonimsky rhythm-drill --measures 2 --seed 7 -o /w/probe.png >/dev/null 2>&1 \
     && [[ -s "$probe_dir/probe.png" ]]; then
    printf '  OK    %-22s %s\n' "scratch: PNG render" "$(du -h "$probe_dir/probe.png" | cut -f1) (font + rasterizer)"
  else
    echo "  FAIL  scratch: PNG render produced nothing" >&2; pfail=1
  fi

  "$RUNTIME" rmi -f rust-music-musl-probe:latest >/dev/null 2>&1 || true
  rm -rf "$probe_dir"; trap - EXIT
  (( pfail == 0 )) || { echo "" >&2; echo "Portability proof FAILED." >&2; exit 1; }
fi

echo ""
echo "Binaries in $BIN_DIR"
echo "Copy into any container image — no libc, no fonts, no shared libraries needed."

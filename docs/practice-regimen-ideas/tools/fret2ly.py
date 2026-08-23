#!/usr/bin/env python3
"""fret2ly — convert (string, fret) guitar positions to LilyPond absolute pitches.

Why this exists: writing LilyPond pitch names by hand for fretboard-specific material is
error-prone. During prototyping, three separate drafts silently produced *unplayable* tab
(frets 17-22 for a 5th-position warmup, 18-22 for comping voicings) because absolute pitch
names don't map intuitively to guitar register. LilyPond compiles them happily.

Always author fretboard material as (string, fret) pairs and convert with this.

Usage:
    # melodic line: string.fret pairs
    ./fret2ly.py 3.3 2.3 2.1 2.4
    -> bes\3 d'\2 c'\2 ees'\2

    # chord voicing: comma-separated within a beat
    ./fret2ly.py --chord 5.3,4.5,3.3,2.5
    -> <c\5 g\4 bes\3 ees'\2>

    # check playability of a span
    ./fret2ly.py --check 3.3 2.3 2.1 2.4
    -> OK  frets 1-4  span 3
"""
import sys

# Standard tuning, open-string MIDI numbers. String 1 = high E.
OPEN = {6: 40, 5: 45, 4: 50, 3: 55, 2: 59, 1: 64}
TUNINGS = {
    "standard": OPEN,
    "drop-d":   {6: 38, 5: 45, 4: 50, 3: 55, 2: 59, 1: 64},
    "dadgad":   {6: 38, 5: 45, 4: 50, 3: 55, 2: 57, 1: 62},
}
# Prefer flats; the caller re-spells per key if needed.
NAMES = {0: "c", 1: "cis", 2: "d", 3: "ees", 4: "e", 5: "f",
         6: "fis", 7: "g", 8: "aes", 9: "a", 10: "bes", 11: "b"}

MAX_FRET = 15   # above this, material is out of scope for this regimen
MAX_SPAN = 4    # one hand position

# Span alone is a bad playability proxy for CHORDS. `x-7-9-5-7-x` (Cmaj7#11) has span 4 and
# passes a span-only check, but it is brutal to hold: the middle of the voicing jumps
# *backwards* 4 frets across a string change (str4 fr9 -> str3 fr5), so the hand has to
# reverse-stretch mid-chord. The same span is fine when the steps are gradual.
# So for chords we also bound the fret change between ADJACENT strings.
MAX_ADJ_DELTA = 3   # fret jump between two adjacent sounded strings
MAX_ADJ_BACK = 2    # a *backwards* jump (toward the nut) going low->high is harder still


def to_ly(midi: int) -> str:
    """MIDI number -> LilyPond absolute pitch (c' == C4 == MIDI 60)."""
    name = NAMES[midi % 12]
    octave = midi // 12 - 1
    delta = octave - 3
    return name + ("'" * delta if delta > 0 else "," * -delta)


def pos_to_ly(string: int, fret: int, tuning=OPEN) -> str:
    if string not in tuning:
        raise SystemExit(f"bad string {string}: expected 1-6")
    if fret < 0:
        raise SystemExit(f"bad fret {fret}")
    return f"{to_ly(tuning[string] + fret)}\\{string}"


def parse(tok: str):
    parts = tok.split(".")
    if len(parts) != 2:
        raise SystemExit(
            f"bad position {tok!r}: expected STRING.FRET (e.g. 3.5). "
            "Quote or space-separate each pair."
        )
    try:
        return int(parts[0]), int(parts[1])
    except ValueError:
        raise SystemExit(f"bad position {tok!r}: string and fret must be integers")


def check(positions, chord=False):
    """Report playability. Returns (ok, message).

    With chord=True also checks the fret change between adjacent sounded strings, which
    catches awkward hand shapes that a span-only check waves through (see MAX_ADJ_* above).
    """
    frets = [f for _, f in positions if f > 0]
    if not frets:
        return True, "OK  (all open strings)"
    lo, hi = min(frets), max(frets)
    span = hi - lo
    problems = []
    if hi > MAX_FRET:
        problems.append(f"fret {hi} > max {MAX_FRET}")
    if span > MAX_SPAN:
        problems.append(f"span {span} > max {MAX_SPAN}")

    detail = ""
    if chord:
        # walk low string -> high string (string numbers descend as pitch rises)
        ordered = sorted(positions, key=lambda sf: -sf[0])
        deltas = []
        for (s1, f1), (s2, f2) in zip(ordered, ordered[1:]):
            if f1 == 0 or f2 == 0:      # open strings impose no stretch
                continue
            deltas.append((s1, s2, f2 - f1))
        for s1, s2, d in deltas:
            if d < 0 and abs(d) > MAX_ADJ_BACK:
                problems.append(
                    f"backwards jump {d} frets across str{s1}->str{s2} "
                    f"(max {MAX_ADJ_BACK} going toward the nut)")
            elif abs(d) > MAX_ADJ_DELTA:
                problems.append(f"jump {d:+d} frets across str{s1}->str{s2} "
                                f"(max {MAX_ADJ_DELTA})")
        if deltas:
            detail = "  Δ=" + ",".join(f"{d:+d}" for _, _, d in deltas)

    if problems:
        return False, "AWKWARD  " + "; ".join(problems) + detail
    return True, f"OK  frets {lo}-{hi}  span {span}{detail}"


def main(argv):
    args = [a for a in argv if not a.startswith("--")]
    flags = {a for a in argv if a.startswith("--")}
    tuning = OPEN
    if not args:
        raise SystemExit(__doc__)

    if "--chord" in flags:
        for group in args:
            positions = [parse(t) for t in group.split(",")]
            ok, msg = check(positions, chord=True)
            pitches = " ".join(pos_to_ly(s, f, tuning) for s, f in positions)
            print(f"<{pitches}>", "   %", msg if not ok else msg)
        return 0

    positions = [parse(t) for t in args]
    ok, msg = check(positions)
    if "--check" in flags:
        print(msg)
        return 0 if ok else 1
    print(" ".join(pos_to_ly(s, f, tuning) for s, f in positions))
    if not ok:
        print(f"% WARNING: {msg}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

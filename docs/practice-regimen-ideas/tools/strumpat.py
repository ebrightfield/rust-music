#!/usr/bin/env python3
"""strumpat — Eric's 2012 "Strum Patterns: A Generalized Approach", as code.

From `()Guitar Lessons/Strum Patterns Generalized Approach.pdf`:

Stroke direction (rules 1-3): with `s` = the smallest subdivision in the pattern, every
odd-numbered occurrence of `s` is a DOWNstroke, every even-numbered one an UPstroke. This is
positional — it depends on where you are in the bar, not on whether you actually strike.

Rule 4: if two/four/six slower-subdivision beats occur in a row, the strumming rate may
temporarily fall to that slower subdivision.

Exhaustive repertoire (rules 1-3 of the second section): with `L` = pattern length measured
in units of `s`, there are exactly `2^L` patterns (each slot struck or not). So the space is
finite and enumerable, which means a generator can pick *unpracticed* ones rather than
recycling the same four.

Rests: for any strum of duration D >= 2s, there are D ways to cut it short for a rest at
subdivision level s.

Usage:
    ./strumpat.py --length 8                    # count + list a few
    ./strumpat.py --length 8 --pick 0b10110110  # one specific pattern, with strokes
    ./strumpat.py --length 8 --sample 5 --seed 3
    ./strumpat.py --length 8 --pick 0b10110110 --lily   # LilyPond rhythm skeleton
"""
import argparse
import random


def strokes(length):
    """Positional stroke directions: odd occurrence = down, even = up (rules 2-3)."""
    return ["D" if i % 2 == 0 else "U" for i in range(length)]


def render(mask, length):
    """mask: int bitfield, bit i set = strike on subdivision i."""
    d = strokes(length)
    hits, dirs = [], []
    for i in range(length):
        struck = bool(mask >> i & 1)
        hits.append("x" if struck else ".")
        dirs.append(d[i] if struck else " ")
    return "".join(hits), "".join(dirs)


def durations(mask, length):
    """Duration of each strike in units of s (a strike lasts until the next strike)."""
    idx = [i for i in range(length) if mask >> i & 1]
    if not idx:
        return []
    out = []
    for j, i in enumerate(idx):
        nxt = idx[j + 1] if j + 1 < len(idx) else length
        out.append((i, nxt - i))
    return out


def to_lily(mask, length, unit=8, chord="q"):
    """Rhythm-only LilyPond: strikes as `chord`, gaps as rests.

    `unit` is the note value of s (8 = eighth). A strike lasting `dur` units is written as
    `dur` tied notes of that value — always rhythmically correct, and LilyPond beams it.
    The caller supplies the actual voicing for the first event and uses `q` (repeat-chord)
    afterwards, so pass chord="<c\\5 g\\4>" for the first strike if needed.
    """
    idx = [i for i in range(length) if mask >> i & 1]
    if not idx:
        return " ".join([f"r{unit}"] * length)
    toks = []
    if idx[0] > 0:
        toks += [f"r{unit}"] * idx[0]
    for i, dur in durations(mask, length):
        toks.append(f"{chord}{unit}")
        toks += [f"~ {chord}{unit}"] * (dur - 1)
    return " ".join(toks)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--length", type=int, default=8,
                    help="pattern length L in units of the smallest subdivision s")
    ap.add_argument("--pick", help="one pattern as 0b… / int / x.x. string")
    ap.add_argument("--sample", type=int, default=0, help="show N random patterns")
    ap.add_argument("--seed", type=int, help="seed for --sample")
    ap.add_argument("--lily", action="store_true", help="emit a LilyPond rhythm skeleton")
    ap.add_argument("--unit", type=int, default=8, help="note value of s (8 = eighth)")
    a = ap.parse_args()

    L = a.length
    print(f"L = {L} subdivisions  ->  2^{L} = {2**L} total patterns")

    def show(mask):
        hits, dirs = render(mask, L)
        print(f"  {hits}   {dirs}   (0b{mask:0{L}b})")
        if a.lily:
            print(f"    lily: {to_lily(mask, L, a.unit)}")

    if a.pick:
        p = a.pick
        if set(p) <= set("x."):
            mask = sum(1 << i for i, c in enumerate(p) if c == "x")
        else:
            mask = int(p, 0)
        show(mask)
        return 0

    if a.sample:
        rng = random.Random(a.seed)
        seen = set()
        while len(seen) < min(a.sample, 2**L):
            m = rng.randrange(1, 2**L)   # exclude all-rests
            if m not in seen:
                seen.add(m)
                show(m)
        return 0

    # default: a few canonical ones
    print("  (examples; use --sample or --pick)")
    for m in [0b10101010, 0b10110110, 0b11011010, 0b10010010]:
        show(m & (2**L - 1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

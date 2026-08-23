#!/usr/bin/env python3
"""scalepat — Eric's 2012 "Making Up Scale Patterns" generator, as code.

From `()Guitar Lessons/Scale Patterns.pdf`:
  1. Pick a non-zero number = how many scale degrees the fragment advances each iteration
     (negative = downward).
  2. Pick a numerical series of any length = the successive intervals (in scale degrees)
     taken within each fragment (negative = downward).
  3. On a starting pitch, move through the scale by the series, then restart from a new
     starting note given by step 1.

Notated in the original as `step,[series]` — e.g. `1,[null]`, `1,[2]`, `-1,[-2]`, `2,[0]`,
`1,[1,1]`. This is a complete generative notation for scalar patterns; the whole point is
that it never runs out.

Usage:
    ./scalepat.py --scale "Bb C D Eb F G A" --step 1 --series 2 --count 8
    ./scalepat.py --scale "G A B D E" --step 1 --series 1 1 --count 6 --degrees
"""
import argparse


def gen(scale_len, step, series, count, start=0):
    """Yield lists of scale-degree indices, one list per fragment."""
    out = []
    idx = start
    for _ in range(count):
        frag = [idx]
        cur = idx
        for iv in series:
            cur += iv
            frag.append(cur)
        out.append(frag)
        idx += step
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scale", required=True,
                    help='space-separated note names, ascending, e.g. "Bb C D Eb F G A"')
    ap.add_argument("--step", type=int, required=True,
                    help="scale degrees to advance per fragment (non-zero; negative = down)")
    ap.add_argument("--series", type=int, nargs="*", default=[],
                    help="interval series in scale degrees; empty = plain run")
    ap.add_argument("--count", type=int, default=8, help="how many fragments")
    ap.add_argument("--start", type=int, default=0, help="starting degree index")
    ap.add_argument("--degrees", action="store_true",
                    help="print degree indices instead of note names")
    a = ap.parse_args()

    if a.step == 0:
        raise SystemExit("--step must be non-zero (see rule 1)")

    scale = a.scale.split()
    frags = gen(len(scale), a.step, a.series, a.count, a.start)

    label = f"{a.step},[{','.join(map(str, a.series)) if a.series else 'null'}]"
    print(f"pattern {label}   scale: {' '.join(scale)}")
    for frag in frags:
        if a.degrees:
            print("  " + " ".join(str(i) for i in frag))
        else:
            # wrap into the scale, tracking octave displacement
            names = []
            for i in frag:
                oct_shift, deg = divmod(i, len(scale))
                names.append(scale[deg] + ("+" * oct_shift if oct_shift > 0
                                           else "-" * -oct_shift))
            print("  " + " ".join(names))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

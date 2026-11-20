#!/usr/bin/env python3
"""Experiment 0007: how much of the old code and proof survives, and what the
change cost relative to the 0006 run that wrote the start file.

    python3 -I survival.py
"""
import difflib
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
EXT = {"dafny": "dfy", "verus": "rs"}
SRC = {"c01": "h01", "c02": "h02", "c03": "h03", "c04": "h04", "c05": "h05", "c06": "h07", "c07": "h08",
       "c08": "h10"}


def after_spec(text):
    lines = text[text.index("// SPEC-END"):].splitlines()[1:]
    return [l.strip() for l in lines if l.strip() and not l.strip().startswith("//")]


def session(path):
    with open(path, encoding="utf-8") as f:
        d = json.load(f)
    calls = len([x for x in os.listdir(os.path.join(os.path.dirname(path), "snap")) if x.endswith(".out")])
    return d.get("total_cost_usd", 0) or 0, calls


def main():
    print("| Task | Verifier | Start lines | Kept | Kept % | Calls (0006 from scratch) | Cost $ (0006) |")
    print("|---|---|---|---|---|---|---|")
    for lang in ("dafny", "verus"):
        for f in sorted(os.listdir(os.path.join(HERE, "tasks", lang))):
            t = f.split("_")[0]
            run = os.path.join(HERE, "runs", lang, t)
            if not os.path.exists(os.path.join(run, "model")):
                continue
            with open(os.path.join(HERE, "tasks", lang, f), encoding="utf-8") as g:
                start = after_spec(g.read())
            with open(os.path.join(run, "final." + EXT[lang]), encoding="utf-8") as g:
                final = after_spec(g.read())
            sm = difflib.SequenceMatcher(a=start, b=final, autojunk=False)
            kept = sum(b.size for b in sm.get_matching_blocks())
            cost, calls = session(os.path.join(run, "session.json"))
            old = os.path.join(HERE, "..", "0006-s6-hard-baseline", "runs", lang, SRC[t], "session.json")
            ocost, ocalls = session(old)
            print("| %s | %s | %d | %d | %.0f%% | %d (%d) | %.3f (%.3f) |" % (
                t, lang, len(start), kept, 100.0 * kept / max(1, len(start)), calls, ocalls, cost, ocost))


if __name__ == "__main__":
    main()

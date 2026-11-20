#!/usr/bin/env python3
"""Experiment 0007: analyze the baseline runs.

    python3 -I analyze.py [--no-stability]

Reads runs/<lang>/<task>/, writes results/runs.tsv, results/errors.tsv,
results/stability.tsv and prints the report tables (Markdown).
"""
import difflib
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.normpath(os.path.join(HERE, "../../../tools/bin"))
EXT = {"dafny": "dfy", "verus": "rs"}

FORBIDDEN = {
    "dafny": [r"\bassume\b", r"\{:axiom\}", r"\{:verify\s+false\}", r"\bexpect\b"],
    "verus": [r"\bassume\s*\(", r"\badmit\s*\(", r"external_body", r"verifier::external\b",
              r"exec_allows_no_decreases_clause"],
}


def read(p):
    with open(p, encoding="utf-8") as f:
        return f.read()


def spec_block(text):
    m = re.search(r"// SPEC-BEGIN\n(.*?)// SPEC-END", text, re.S)
    return m.group(1) if m else None


def verify(lang, path):
    d, f = os.path.split(path)
    if lang == "dafny":
        cmd = [os.path.join(TOOLS, "dafny"), "verify", f]
    else:
        cmd = [os.path.join(TOOLS, "verus"), f]
    try:
        r = subprocess.run(cmd, cwd=d, capture_output=True, text=True, timeout=300)
        out = r.stdout + r.stderr
    except subprocess.TimeoutExpired:
        out = "TIMEOUT"
    return "\n".join(l for l in out.splitlines() if not l.startswith("Prover error"))


def verified(lang, out):
    if lang == "dafny":
        return bool(re.search(r"verifier finished with \d+ verified, 0 errors", out)) and \
            not re.search(r"\bError\b", out)
    return bool(re.search(r"verification results:: \d+ verified, 0 errors", out)) and \
        not re.search(r"^error", out, re.M)


def error_class(lang, out):
    """Class of the first error in a failing verifier output."""
    if "TIMEOUT" in out:
        return "verification"
    if lang == "dafny":
        if re.search(r"parse errors? detected", out):
            return "syntax"
        if re.search(r"resolution/type errors? detected", out):
            return "typing"
        if re.search(r"might be uninitialized", out) and not re.search(r"could not be proved|might not", out):
            return "typing"
        return "verification"
    m = re.search(r"^error(\[E\d+\])?: (.*)$", out, re.M)
    if not m:
        return "verification" if "errors" in out else "other"
    code, msg = m.group(1), m.group(2)
    if re.search(r"unsupported|not supported|not yet supported|cannot call function .* with mode exec"
                 r"|cannot use function .* which is marked|is not supported", msg):
        return "false friend"
    if code:
        return "typing"
    if re.search(r"^expected |unexpected |mismatched closing|unclosed delimiter|this file contains an un", msg):
        return "syntax"
    if re.search(r"not satisfied|assertion failed|possible arithmetic|might not|could not automatically infer"
                 r"|trigger|rlimit|timed out|decreases|loop invariant|not hold", msg):
        return "verification"
    return "typing"


def matching_brace(text, i):
    """Index just after the brace that closes the '{' at text[i]."""
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return j + 1
    return len(text)


def body_span(text):
    s = text.index("// SPEC-END")
    b = text.index("{", s)
    return b, matching_brace(text, b)


def classify_lines(lang, task_text, final_text):
    """Counts of added non-comment lines: (code, annotation, proof step)."""
    added = set()
    sm = difflib.SequenceMatcher(a=task_text.splitlines(), b=final_text.splitlines(), autojunk=False)
    for tag, _, _, j1, j2 in sm.get_opcodes():
        if tag in ("replace", "insert"):
            added.update(range(j1, j2))
    lines = final_text.splitlines()
    # Proof regions: lemma / proof fn bodies, proof blocks, calc, assert-by bodies.
    proof = set()
    header = set()
    text = final_text
    offsets = [0]
    for l in lines:
        offsets.append(offsets[-1] + len(l) + 1)

    def line_of(pos):
        lo, hi = 0, len(lines)
        while lo < hi:
            mid = (lo + hi) // 2
            if offsets[mid + 1] <= pos:
                lo = mid + 1
            else:
                hi = mid
        return lo

    if lang == "dafny":
        starts = [r"^\s*(ghost\s+)?lemma\b", r"\bcalc\b", r"\bassert\b[^;]*\bby\b"]
    else:
        starts = [r"^\s*(pub\s+)?proof\s+fn\b", r"\bproof\s*\{", r"\bassert\b[^;]*\bby\b"]
    lemma_names = set(re.findall(r"\blemma\s+(?:\{[^}]*\}\s*)?(\w+)", text) if lang == "dafny"
                      else re.findall(r"\bproof\s+fn\s+(\w+)", text))
    for pat in starts:
        for m in re.finditer(pat, text, re.M):
            b = text.find("{", m.start())
            if b < 0:
                continue
            e = matching_brace(text, b)
            lb, le = line_of(b), line_of(e - 1)
            for k in range(line_of(m.start()), lb):
                header.add(k)
            for k in range(lb + 1, le):
                proof.add(k)
            if "{" in lines[lb][lines[lb].index("{") + 1:] or lines[lb].rstrip().endswith("{") is False:
                pass
    annot_re = (r"^\s*(invariant|decreases|assert\b|ghost\b|requires|ensures|reads|modifies|predicate|function"
                r"|lemma|reveal|twostate)" if lang == "dafny" else
                r"^\s*(invariant|decreases|assert\b|ghost\b|tracked\b|requires|ensures|(pub\s+)?(open\s+)?"
                r"(spec|proof)\s+fn|let\s+ghost|reveal|broadcast|forall\||exists\||#\[trigger\])")
    code = annot = step = 0
    for k in sorted(added):
        s = lines[k].strip()
        if not s or s.startswith("//") or s in ("{", "}", "},", "};"):
            continue
        if k in proof:
            if re.match(r"^\s*(invariant|decreases)", s):
                annot += 1
            else:
                step += 1
        elif k in header or re.match(annot_re, lines[k]):
            annot += 1
        elif any(re.match(r"^%s\s*(::<[^>]*>)?\(" % re.escape(n), s) for n in lemma_names):
            annot += 1
        elif re.match(r"^\s*(&&|\|\||==>|<==>)", lines[k]) or re.match(r"^\s*[^=;]*(<=|>=|<|>|==)[^=;]*,\s*$", lines[k]):
            # continuation of a multi-line clause (Verus invariant lists, Dafny conjunctions)
            annot += 1
        else:
            code += 1
    return code, annot, step


UNRELATED = {
    "dafny": "\nmethod unrelated_q(a: u32) returns (b: int)\n  ensures b == a + 1\n{\n  b := a + 1;\n}\n",
    "verus": "fn unrelated_q(a: u32) -> (b: u32)\n    requires\n        a < 100,\n    ensures\n"
             "        b == a + 1,\n{\n    a + 1\n}\n\n",
}


def edits(lang, text):
    """Mechanical semantics-preserving edits: {name: new text or None if not applicable}."""
    b, e = body_span(text)
    body = text[b:e]
    res = {}
    m = re.search(r"\bvar\s+(\w+)" if lang == "dafny" else r"\blet\s+(?:(?:mut|ghost|tracked)\s+)*(\w+)", body)
    if m and m.group(1) != "_":
        v = m.group(1)
        res["E1"] = text[:b] + re.sub(r"\b%s\b" % re.escape(v), v + "_q", body) + text[e:]
    else:
        res["E1"] = None
    lines = body.split("\n")
    res["E2"] = None
    if lang == "dafny":
        for i in range(len(lines) - 2):
            a, c, nxt = lines[i].strip(), lines[i + 1].strip(), lines[i + 2].strip()
            if a.startswith("invariant") and c.startswith("invariant") and \
                    re.match(r"(invariant|decreases|modifies|\{)", nxt):
                lines[i], lines[i + 1] = lines[i + 1], lines[i]
                res["E2"] = text[:b] + "\n".join(lines) + text[e:]
                break
    else:
        for i in range(len(lines) - 3):
            if lines[i].strip() == "invariant":
                a, c, nxt = lines[i + 1].strip(), lines[i + 2].strip(), lines[i + 3].strip()
                if a.endswith(",") and c.endswith(",") and not re.match(r"(decreases|ensures)", c) and (nxt.endswith(",") or
                                                           re.match(r"(decreases|ensures|\{)", nxt)):
                    lines[i + 1], lines[i + 2] = lines[i + 2], lines[i + 1]
                    res["E2"] = text[:b] + "\n".join(lines) + text[e:]
                break
    if lang == "dafny":
        res["E3"] = text + UNRELATED["dafny"]
    else:
        i = text.rindex("fn main()")
        res["E3"] = text[:i] + UNRELATED["verus"] + text[i:]
    return res


def main():
    do_stab = "--no-stability" not in sys.argv
    os.makedirs(os.path.join(HERE, "results"), exist_ok=True)
    rows, errs, stab = [], [], []
    for lang in ("dafny", "verus"):
        tdir = os.path.join(HERE, "tasks", lang)
        for tf in sorted(os.listdir(tdir)):
            task = tf.split("_")[0]
            rdir = os.path.join(HERE, "runs", lang, task)
            if not os.path.exists(os.path.join(rdir, "model")):  # written last by run.sh
                continue
            task_text = read(os.path.join(tdir, tf))
            final_text = read(os.path.join(rdir, "final." + EXT[lang]))
            final_out = read(os.path.join(rdir, "final.out"))
            try:
                sess = json.loads(read(os.path.join(rdir, "session.json")))
            except ValueError:
                sess = {}
            ok_verify = verified(lang, final_out)
            ok_spec = spec_block(task_text) == spec_block(final_text)
            bad = [p for p in FORBIDDEN[lang] if re.search(p, re.sub(r"//.*", "", final_text))]
            review_path = os.path.join(rdir, "review.txt")
            review = read(review_path).strip() if os.path.exists(review_path) else ""
            ok_review = not review.startswith("FAIL")
            success = ok_verify and ok_spec and not bad and ok_review
            snaps = sorted((f for f in os.listdir(os.path.join(rdir, "snap")) if f.endswith(".out")),
                           key=lambda f: int(f.split(".")[0]))
            classes = []
            for s in snaps:
                out = read(os.path.join(rdir, "snap", s))
                if not verified(lang, out):
                    c = error_class(lang, out)
                    classes.append(c)
                    errs.append((lang, task, s.split(".")[0], c))
            u = sess.get("usage", {})
            tin = u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0) + \
                u.get("cache_read_input_tokens", 0)
            code, annot, step = classify_lines(lang, task_text, final_text)
            rows.append(dict(
                lang=lang, task=task, success=int(success), verify_ok=int(ok_verify), spec_ok=int(ok_spec),
                forbidden=";".join(bad), review=review, calls=len(snaps),
                turns=sess.get("num_turns", ""), tokens_in=tin, tokens_out=u.get("output_tokens", 0),
                cost=round(sess.get("total_cost_usd", 0) or 0, 4),
                wall=read(os.path.join(rdir, "wall_seconds")).strip(),
                code=code, annot=annot, step=step, claimed=(sess.get("result") or "").strip()[-40:],
                errors=",".join(classes)))
            if do_stab and success:
                for name, new in edits(lang, final_text).items():
                    if new is None:
                        stab.append((lang, task, name, "n/a"))
                        continue
                    with tempfile.TemporaryDirectory() as d:
                        p = os.path.join(d, "t." + EXT[lang])
                        with open(p, "w", encoding="utf-8") as f:
                            f.write(new)
                        stab.append((lang, task, name, "pass" if verified(lang, verify(lang, p)) else "FAIL"))
    keys = list(rows[0].keys()) if rows else []
    with open(os.path.join(HERE, "results", "runs.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(keys) + "\n")
        for r in rows:
            f.write("\t".join(str(r[k]) for k in keys) + "\n")
    with open(os.path.join(HERE, "results", "errors.tsv"), "w", encoding="utf-8") as f:
        f.write("lang\ttask\tcall\tclass\n")
        for e in errs:
            f.write("\t".join(e) + "\n")
    if do_stab:
        with open(os.path.join(HERE, "results", "stability.tsv"), "w", encoding="utf-8") as f:
            f.write("lang\ttask\tedit\tresult\n")
            for s in stab:
                f.write("\t".join(s) + "\n")

    print("| Task | Verifier | Success | Calls | Turns | Tokens in / out | Cost $ | Wall s | Code / annot / step lines | Errors by call |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        print("| %s | %s | %s | %s | %s | %s / %s | %s | %s | %s / %s / %s | %s |" % (
            r["task"], r["lang"], "yes" if r["success"] else "**no** (%s)" % ",".join(
                x for x, b in (("verify", not r["verify_ok"]), ("spec", not r["spec_ok"]),
                               ("forbidden", bool(r["forbidden"])), ("review", r["review"].startswith("FAIL")))
                if b), r["calls"], r["turns"], r["tokens_in"], r["tokens_out"], r["cost"], r["wall"],
            r["code"], r["annot"], r["step"], r["errors"] or "—"))
    print()
    for lang in ("dafny", "verus"):
        rs = [r for r in rows if r["lang"] == lang]
        if not rs:
            continue
        cs = [e[3] for e in errs if e[0] == lang]
        lines = sum(r["code"] + r["annot"] + r["step"] for r in rs if r["success"])
        steps = sum(r["step"] for r in rs if r["success"])
        annots = sum(r["annot"] for r in rs if r["success"])
        print("%s: %d/%d solved; calls total %d; cost $%.2f; errors %s; added lines in solved runs %d "
              "(annotation %d, proof step %d = %.0f%% of non-code)" % (
                  lang, sum(r["success"] for r in rs), len(rs), sum(r["calls"] for r in rs),
                  sum(r["cost"] for r in rs), {c: cs.count(c) for c in sorted(set(cs))}, lines, annots,
                  steps, 100.0 * steps / max(1, annots + steps)))
        st = [s for s in stab if s[0] == lang and s[3] != "n/a"]
        if st:
            print("  stability: %d/%d edits broke verification: %s" % (
                sum(s[3] == "FAIL" for s in st), len(st),
                ", ".join("%s %s" % (s[1], s[2]) for s in st if s[3] == "FAIL") or "none"))


if __name__ == "__main__":
    main()

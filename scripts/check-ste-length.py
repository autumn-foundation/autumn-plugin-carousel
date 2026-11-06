"""Check that prose sentences have 25 words or fewer (ASD-STE100).

Reads Markdown files and `//!`/`///`/`//`/`/* */` comments. Skips code
blocks, tables, headings and link-only lines.
"""
import re, sys

def prose_md(text):
    out, code = [], False
    for line in text.splitlines():
        if line.strip().startswith("```"):
            code = not code; continue
        if code or line.lstrip().startswith(("|", "#", "<!--")): continue
        out.append(line)
    return "\n".join(out)

def prose_code(text):
    out = []
    for line in text.splitlines():
        m = re.match(r"\s*(//[!/]?|\*|/\*)\s?(.*)", line)
        if m: out.append(m.group(2).rstrip("*/ "))
        else: out.append("")
    return "\n".join(out)

LIMIT = 25
bad = 0
if len(sys.argv) < 2:
    sys.exit("usage: check-ste-length.py FILE...")
for path in sys.argv[1:]:
    text = open(path, encoding="utf-8").read()
    prose = prose_md(text if path.endswith(".md") else prose_code(text))
    # Join wrapped lines into paragraphs; list items start new units.
    paras = re.split(r"\n\s*\n|\n\s*[-*] |\n\s*\d+\. ", prose)
    for para in paras:
        flat = re.sub(r"`[^`]*`", "X", " ".join(para.split()))
        flat = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", flat)
        for sentence in re.split(r"(?<=[.!?:])\s+(?=[A-Z`\"(])", flat):
            words = [w for w in sentence.split() if re.search(r"[A-Za-z0-9]", w)]
            if len(words) > LIMIT:
                bad += 1
                print(f"{path}: {len(words)} words: {sentence[:140]}")
print(f"{bad} sentence(s) over {LIMIT} words")
sys.exit(1 if bad else 0)

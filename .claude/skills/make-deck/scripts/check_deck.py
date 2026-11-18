#!/usr/bin/env python3
"""Check a make-deck Markdown deck against its source doc. Standard library only.

    check_deck.py deck.md                 the source is the front matter's `source:` path
    check_deck.py deck.md --source x.md   or the path given here

Errors are what references/deck-spec.md forbids; warnings are worth a look. It prints one
line per finding and a last line `N errors, M warnings`, and exits 1 on any error.

What it checks:
- front matter with `title` and `source`, and a source file that exists;
- a title slide first, with at most two lines under its `# `, and one `## ` title
  on every other slide;
- the slide budget: main slides (before a slide titled Backup or Appendix) within `slides:`
  from the front matter, else 13;
- per slide: at most 6 bullets, at most 45 words of body text outside code and tables,
  code blocks of at most 14 lines, tables of at most 7 rows and 4 columns;
- fidelity: every number of two or more digits, and every requirement ID (FR3, NFR11, LLD1),
  appears in the source, so the deck states nothing the doc does not;
- no em or en dashes in slide text;
- every image path exists.
"""
import argparse
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from render_deck import parse_deck, slide_title  # noqa: E402

MAX_BULLETS, MAX_WORDS, MAX_CODE, MAX_ROWS, MAX_COLS = 6, 45, 14, 7, 4
DEFAULT_SLIDES = 13   # 20 minutes, the default talk
BACKUP = re.compile(r"^(backup|appendix)\b", re.I)
NUMBER = re.compile(r"(?<![\w.])\d[\d,]*(?:\.\d+)?")
REQ_ID = re.compile(r"\b(?:FR|NFR|LLD)\d+\b")
IMAGE = re.compile(r"!\[[^\]]*\]\(([^)\s]+)\)")


def strip_code_and_tables(body):
    """Return (prose_lines, code_blocks, tables): prose excludes fenced code and table rows."""
    prose, code, tables, block, table, fenced = [], [], [], [], [], False
    for line in body.splitlines():
        if line.lstrip().startswith("```"):
            if fenced:
                code.append(block)
                block = []
            fenced = not fenced
            continue
        if fenced:
            block.append(line)
            continue
        if line.lstrip().startswith("|"):
            table.append(line)
            continue
        if table:
            tables.append(table)
            table = []
        prose.append(line)
    if table:
        tables.append(table)
    return prose, code, tables


def numbers_in(text):
    return {n.replace(",", "") for n in NUMBER.findall(text)}


def check(deck_path, source_path=None):
    errors, warnings = [], []
    deck_path = Path(deck_path)
    fields, slides = parse_deck(deck_path.read_text(encoding="utf-8"))

    for key in ("title", "source"):
        if not fields.get(key):
            errors.append(f"front matter: no `{key}:` line")
    source = Path(source_path) if source_path else None
    if source is None and fields.get("source"):
        for base in [deck_path.parent] + list(deck_path.resolve().parents):
            if (base / fields["source"]).exists():
                source = base / fields["source"]
                break
    if source is None or not source.exists():
        errors.append(f"source not found: {fields.get('source') or source_path}")
        source_text = None
    else:
        source_text = source.read_text(encoding="utf-8")
    src_numbers = numbers_in(source_text) if source_text else set()
    src_ids = set(REQ_ID.findall(source_text)) if source_text else set()

    if not slides:
        errors.append("the deck has no slides")
        return errors, warnings
    if not re.match(r"^\s*#\s", slides[0]["body"]):
        errors.append("slide 1: the first slide is the title slide, opening with one `# ` heading")
    else:
        extra = [l for l in slides[0]["body"].splitlines() if l.strip() and not re.match(r"^\s*#\s", l)]
        if len(extra) > 2:
            errors.append(f"slide 1: {len(extra)} lines under the title; the limit is 2")

    try:
        budget = int(fields.get("slides", DEFAULT_SLIDES))
    except ValueError:
        budget = DEFAULT_SLIDES
        warnings.append(f"front matter: `slides:` is not a number; using {DEFAULT_SLIDES}")
    main, in_backup = 0, False
    for n, slide in enumerate(slides, 1):
        where = f"slide {n}"
        body, notes = slide["body"], slide["notes"]
        title = slide_title(body)
        if n > 1:
            heads = re.findall(r"^##\s+(.+)$", body, re.M)
            if len(heads) != 1:
                errors.append(f"{where}: needs exactly one `## ` title, has {len(heads)}")
            if re.search(r"^#\s", body, re.M):
                errors.append(f"{where}: `# ` belongs to the title slide only")
        if BACKUP.match(title or ""):
            in_backup = True
        if not in_backup:
            main += 1

        prose, code, tables = strip_code_and_tables(body)
        text = [l for l in prose if not re.match(r"^\s*#{1,3}\s", l)]
        bullets = sum(1 for l in text if re.match(r"^\s*([-*]|\d+\.)\s+", l))
        if bullets > MAX_BULLETS:
            errors.append(f"{where}: {bullets} bullets; the limit is {MAX_BULLETS}")
        words = len(re.sub(r"!\[[^\]]*\]\([^)]*\)", "", " ".join(text)).split())
        if words > MAX_WORDS:
            errors.append(f"{where}: {words} words outside code and tables; the limit is {MAX_WORDS}")
        for block in code:
            if len(block) > MAX_CODE:
                errors.append(f"{where}: a code block of {len(block)} lines; the limit is {MAX_CODE}")
        for table in tables:
            rows = [r for r in table if not re.fullmatch(r"\s*\|?[\s:|-]+\|?\s*", r)]
            cols = max(len(r.strip().strip("|").split("|")) for r in rows) if rows else 0
            if len(rows) > MAX_ROWS:
                errors.append(f"{where}: a table of {len(rows)} rows; the limit is {MAX_ROWS}")
            if cols > MAX_COLS:
                errors.append(f"{where}: a table of {cols} columns; the limit is {MAX_COLS}")
        if not body.strip() or (n > 1 and not "".join(text[1:]).strip() and not code and not tables
                                and not IMAGE.search(body)):
            warnings.append(f"{where}: no content below the title")

        visible = body
        for ch, name in (("—", "em dash"), ("–", "en dash")):
            if ch in visible or ch in notes:
                errors.append(f"{where}: an {name}; use a period, comma, colon or parentheses")
        if source_text is not None:
            scan = body + "\n" + notes
            for number in sorted(numbers_in(scan)):
                if len(number.replace(".", "")) >= 2 and number not in src_numbers:
                    errors.append(f"{where}: the number {number} is not in the source")
            for rid in sorted(set(REQ_ID.findall(scan))):
                if rid not in src_ids:
                    errors.append(f"{where}: {rid} is not in the source")
        for target in IMAGE.findall(body):
            if not (deck_path.parent / target).exists():
                errors.append(f"{where}: image not found: {target}")
        if n > 1 and not notes and not in_backup:
            warnings.append(f"{where}: no speaker notes")

    if main > budget:
        errors.append(f"{main} main slides; the budget is {budget}. Cut, merge, or move some to Backup")
    return errors, warnings


def main():
    ap = argparse.ArgumentParser(description="Check a make-deck deck against its source.")
    ap.add_argument("deck")
    ap.add_argument("--source")
    a = ap.parse_args()
    if not Path(a.deck).exists():
        print(f"check_deck: no such file: {a.deck}")
        return 2
    errors, warnings = check(a.deck, a.source)
    for e in errors:
        print(f"  error  {e}")
    for w in warnings:
        print(f"  warn   {w}")
    print(f"{len(errors)} errors, {len(warnings)} warnings")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

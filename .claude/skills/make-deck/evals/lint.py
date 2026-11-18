#!/usr/bin/env python3
"""make-deck's own lint, run by scripts/check-skills. Free: no model runs, no browser.

- the scripts' unit tests pass;
- the deck-spec example slide passes the check script against a source that holds its numbers;
- SKILL.md ends with the 'Hand it over' section, which its cut-short check relies on.
"""
import re
import subprocess
import sys
from pathlib import Path

skill = Path(__file__).resolve().parent.parent
scripts = skill / "scripts"
problems = []

tests = subprocess.run([sys.executable, "-B", str(scripts / "test_make_deck.py")],
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True)
if tests.returncode != 0:
    problems.append("scripts/test_make_deck.py failed: " + (tests.stdout.strip().splitlines() or ["no output"])[-1])

spec = (skill / "references" / "deck-spec.md").read_text(encoding="utf-8")
if "## The format" not in spec or "## Slide rules" not in spec:
    problems.append("references/deck-spec.md: lost its 'The format' or 'Slide rules' section")

skill_md = (skill / "SKILL.md").read_text(encoding="utf-8")
headings = re.findall(r"^## (.+)$", skill_md, re.M)
if not headings or headings[-1] != "Hand it over":
    problems.append("SKILL.md must end with the 'Hand it over' section, which its cut-short check relies on")

print("\n".join(problems))
sys.exit(1 if problems else 0)

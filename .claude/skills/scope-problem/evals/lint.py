#!/usr/bin/env python3
"""scope-problem's own lint, run by scripts/check-skills.

The resume table reads progress lines from scoping.md. Each line must be named
the same way in three places: the resume table (SKILL.md), the progress-line
table (requirement-spec.md), and the instruction that adds it (SKILL.md or
sign-off.md). A line that is tested but never written sends every resume to
the wrong phase.
"""
import re
import sys
from pathlib import Path

skill = Path(__file__).resolve().parent.parent
skill_md = (skill / "SKILL.md").read_text(encoding="utf-8")
spec = (skill / "requirement-spec.md").read_text(encoding="utf-8")
sign_off = (skill / "sign-off.md").read_text(encoding="utf-8")

resume = re.findall(r"^\| `([A-Z][A-Za-z ]+:)` \|", skill_md, re.M)
in_spec = re.findall(r"^\| `([A-Z][A-Za-z ]+:) <date>", spec, re.M)
written = set(re.findall(r"[Aa]dd `([A-Z][A-Za-z ]+:) <date>", skill_md + sign_off))

problems = []
if not resume:
    problems.append("SKILL.md: no progress lines found in the resume table")
if resume != in_spec:
    problems.append(f"resume table lists {resume}, but requirement-spec.md lists {in_spec}")
for line in resume:
    if line not in written:
        problems.append(f"`{line}` is tested on resume, but no phase says to add it")
for name in ("interview.md", "sign-off.md", "prompts.md", "requirement-spec.md"):
    if f"({name})" not in skill_md:
        problems.append(f"SKILL.md never links {name}")
headings = re.findall(r"^## (.+)$", skill_md, re.M)
if not headings or headings[-1] != "Hand off":
    problems.append("SKILL.md must end with the 'Hand off' section, which its cut-short check relies on")

print("\n".join(problems))
sys.exit(1 if problems else 0)

#!/usr/bin/env python3
"""draw-diagram's own lint, run by scripts/check-skills. Free: no model runs.

The skill renders nothing, so scripts/check_excalidraw.py stands in for looking
at the picture. Two things keep that honest:

- the script's unit tests pass;
- every JSON template in references/element-templates.md, which the skill tells
  the model to copy, loads in the script with no error.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

skill = Path(__file__).resolve().parent.parent
scripts = skill / "scripts"
problems = []

tests = subprocess.run([sys.executable, "-B", str(scripts / "test_check_excalidraw.py")],
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True)
if tests.returncode != 0:
    problems.append("scripts/test_check_excalidraw.py failed: " + (tests.stdout.strip().splitlines() or ["no output"])[-1])

sys.dont_write_bytecode = True
sys.path.insert(0, str(scripts))
import check_excalidraw  # noqa: E402

templates = (skill / "references" / "element-templates.md").read_text(encoding="utf-8")
heading = "the top of the file"
blocks = 0
for match in re.finditer(r"^#{2,3} ([^\n]+)$|^```json\n(.*?)^```", templates, re.M | re.S):
    if match.group(1):
        heading = match.group(1)
        continue
    blocks += 1
    try:
        elements = json.loads(match.group(2))
    except ValueError as err:
        problems.append(f"element-templates.md, '{heading}': the template is not JSON ({err})")
        continue
    errors, _ = check_excalidraw.check({"type": "excalidraw", "version": 2, "elements": elements})
    problems += [f"element-templates.md, '{heading}': {line}" for line in errors]
if not blocks:
    problems.append("element-templates.md: no json templates found")

skill_md = (skill / "SKILL.md").read_text(encoding="utf-8")
headings = re.findall(r"^## (.+)$", skill_md, re.M)
if not headings or headings[-1] != "Hand it over":
    problems.append("SKILL.md must end with the 'Hand it over' section, which its cut-short check relies on")

print("\n".join(problems))
sys.exit(1 if problems else 0)

#!/usr/bin/env python3
"""Unit tests for check_deck.py and render_deck.py's parsing and HTML. No browser needed."""
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_deck  # noqa: E402
import render_deck  # noqa: E402

SOURCE = """# RFC: Thing
| NFR3 | Redirects answer within 100 ms at p99. |
| FR3 | Equal URLs share one code. |
Peak is 11,600 creations a second; retries at most 5 times.
"""

GOOD = """---
title: Thing
source: rfc.md
slides: 4
---

# Thing

One sentence thesis

---

## Redirects stay local, so they answer in 100 ms (NFR3)

- **Cache** first, then the local replica
- Peak is 11,600 creations a second

Notes: Overview.

---

## Equal URLs share one code (FR3)

```text
Phase 1, look up
```

Notes: Link Creation.
"""


def run(deck, source=SOURCE):
    d = Path(tempfile.mkdtemp())
    (d / "rfc.md").write_text(source, encoding="utf-8")
    (d / "deck.md").write_text(deck, encoding="utf-8")
    return check_deck.check(d / "deck.md")


class CheckDeck(unittest.TestCase):
    def test_good_deck_is_clean(self):
        errors, warnings = run(GOOD)
        self.assertEqual(errors, [])
        self.assertEqual(warnings, [])

    def test_number_not_in_source(self):
        errors, _ = run(GOOD.replace("11,600", "12,000"))
        self.assertTrue(any("12000 is not in the source" in e for e in errors), errors)

    def test_requirement_id_not_in_source(self):
        errors, _ = run(GOOD.replace("(FR3)", "(FR9)"))
        self.assertTrue(any("FR9 is not in the source" in e for e in errors), errors)

    def test_single_digits_are_not_checked(self):
        errors, _ = run(GOOD.replace("Phase 1", "Phase 7"))
        self.assertEqual(errors, [])

    def test_too_many_bullets(self):
        bullets = "\n".join(f"- point {i}" for i in range(1, 8))
        errors, _ = run(GOOD.replace("- **Cache** first, then the local replica", bullets))
        self.assertTrue(any("8 bullets" in e for e in errors), errors)

    def test_too_many_words(self):
        errors, _ = run(GOOD.replace("then the local replica", "word " * 50))
        self.assertTrue(any("words outside code" in e for e in errors), errors)

    def test_dash_is_an_error(self):
        errors, _ = run(GOOD.replace("first, then", "first — then"))
        self.assertTrue(any("em dash" in e for e in errors), errors)

    def test_missing_title_and_two_titles(self):
        errors, _ = run(GOOD.replace("## Equal URLs share one code (FR3)", "Equal URLs"))
        self.assertTrue(any("exactly one `## ` title, has 0" in e for e in errors), errors)

    def test_over_budget_but_backup_is_free(self):
        errors, _ = run(GOOD.replace("slides: 4", "slides: 2"))
        self.assertTrue(any("3 main slides" in e for e in errors), errors)
        backup = GOOD.replace("## Equal URLs share one code (FR3)", "## Backup: equal URLs (FR3)")
        errors, _ = run(backup.replace("slides: 4", "slides: 2"))
        self.assertEqual(errors, [])

    def test_missing_image(self):
        errors, _ = run(GOOD.replace("Notes: Overview.", "![arch](nope.svg)\n\nNotes: Overview."))
        self.assertTrue(any("image not found" in e for e in errors), errors)

    def test_missing_source(self):
        errors, _ = run(GOOD.replace("source: rfc.md", "source: gone.md"))
        self.assertTrue(any("source not found" in e for e in errors), errors)

    def test_title_slide_lines(self):
        errors, _ = run(GOOD.replace("One sentence thesis", "a\nb\nc"))
        self.assertTrue(any("3 lines under the title" in e for e in errors), errors)


class Render(unittest.TestCase):
    def test_separator_inside_code_is_not_a_slide_break(self):
        _, slides = render_deck.parse_deck("---\ntitle: T\n---\n# T\n---\n## A\n```\n---\n```\n")
        self.assertEqual(len(slides), 2)

    def test_notes_are_split_off(self):
        _, slides = render_deck.parse_deck("# T\n---\n## A\n- x\n\nNotes: secret\nmore\n")
        self.assertEqual(slides[1]["notes"], "secret\nmore")
        self.assertNotIn("secret", slides[1]["body"])

    def test_inline_bold_around_code_and_escaping(self):
        out = render_deck.inline("**A `B` C** <x> `a*b*`")
        self.assertEqual(out, "<strong>A <code>B</code> C</strong> &lt;x&gt; <code>a*b*</code>")

    def test_nested_list_html_is_balanced(self):
        out = render_deck.render_list(["- a", "  - b", "  - c", "- d"])
        self.assertEqual(out.count("<ul>"), out.count("</ul>"))
        self.assertEqual(out.count("<li>"), out.count("</li>"))

    def test_html_has_one_section_per_slide_and_no_notes(self):
        fields, slides = render_deck.parse_deck(GOOD)
        html = render_deck.build_html(fields, slides, Path("."), tempfile.mkdtemp(), [])
        self.assertEqual(html.count("<section"), 3)
        self.assertNotIn("Link Creation.", html)

    def test_mermaid_without_mmdc_falls_back_to_source(self):
        warnings = []
        out = render_deck.render_mermaid("sequenceDiagram\nA->>B: x", tempfile.mkdtemp(), 1, warnings) \
            if not render_deck.shutil.which("mmdc") else "Mermaid source"
        self.assertIn("Mermaid source", out)


if __name__ == "__main__":
    unittest.main(verbosity=0)

#!/usr/bin/env python3
"""Unit tests for check_deck.py and render_deck.py's parsing and HTML. No browser needed."""
import os
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

    def test_mermaid_has_its_own_limits(self):
        diagram = "```mermaid\nflowchart LR\n" + "\n".join(f"  A{i} --> A{i+1}" for i in range(20)) + "\n```"
        slide = "\n---\n\n## The flow\n\n" + diagram + "\n\nNotes: Overview.\n"
        errors, _ = run(GOOD + slide)
        self.assertEqual(errors, [])
        long = diagram.replace("```\n", "", 0).replace("\n```", "\n" + "\n".join(f"  B{i} --> B{i+1}" for i in range(10)) + "\n```")
        errors, _ = run(GOOD + slide.replace(diagram, long))
        self.assertTrue(any("Mermaid diagram of" in e for e in errors), errors)
        wordy = slide.replace("## The flow", "## The flow\n\n- " + "word " * 25)
        errors, _ = run(GOOD + wordy)
        self.assertTrue(any("beside a diagram" in e for e in errors), errors)

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

    def test_mermaid_block_becomes_a_slot_holding_its_source(self):
        out = render_deck.render_mermaid("sequenceDiagram\nA->>B: x", tempfile.mkdtemp(), 1, [])
        self.assertIn('<div class="diagram">', out)
        self.assertIn("A-&gt;&gt;B: x", out)

    def test_no_local_copy_uses_the_pinned_cdn_with_its_hash(self):
        old = os.environ.pop("MERMAID_JS", None)
        saved, render_deck.MERMAID_CACHE = render_deck.MERMAID_CACHE, Path(tempfile.mkdtemp()) / "none.js"
        try:
            src, integrity = render_deck.mermaid_source([])
        finally:
            render_deck.MERMAID_CACHE = saved
            if old is not None:
                os.environ["MERMAID_JS"] = old
        self.assertIn("mermaid@" + render_deck.MERMAID_VERSION, src)
        self.assertEqual(integrity, "sha384-" + render_deck.MERMAID_SHA384)

    def test_a_local_copy_with_the_wrong_hash_is_not_used(self):
        fake = Path(tempfile.mkdtemp()) / "mermaid.min.js"
        fake.write_text("window.mermaid = {};", encoding="utf-8")
        old = os.environ.get("MERMAID_JS")
        os.environ["MERMAID_JS"] = str(fake)
        warnings = []
        try:
            src, integrity = render_deck.mermaid_source(warnings)
        finally:
            if old is None:
                del os.environ["MERMAID_JS"]
            else:
                os.environ["MERMAID_JS"] = old
        self.assertTrue(src.startswith("https://"))
        self.assertTrue(any("hash mismatch" in w for w in warnings), warnings)

    def test_no_script_tag_without_a_diagram(self):
        fields, slides = render_deck.parse_deck(GOOD)
        page = render_deck.build_html(fields, slides, Path("."), tempfile.mkdtemp(), [])
        self.assertNotIn("<script src=", page)


class Overflow(unittest.TestCase):
    """Needs a local Chrome or Chromium; skipped without one."""

    def test_a_long_code_block_is_reported(self):
        # Regression: a code block shrank to fit the slide and clipped its own last
        # lines, so the overflow check passed a slide that had lost content.
        chrome = render_deck.find_chrome()
        if not chrome:
            self.skipTest("no Chrome or Chromium")
        code = "\n".join(f"line {i}" for i in range(30))
        deck = f"# T\n---\n## Fits\n- one\n---\n## Long code\n```\n{code}\n```\n"
        fields, slides = render_deck.parse_deck(deck)
        d = Path(tempfile.mkdtemp())
        page = d / "deck.html"
        page.write_text(render_deck.build_html(fields, slides, d, str(d), []), encoding="utf-8")
        self.assertEqual(render_deck.overflowing(chrome, page), [3])


class Diagrams(unittest.TestCase):
    """Needs a local Chrome or Chromium. The drawing test also needs MERMAID_JS set to a
    verified copy of the pinned Mermaid; without one it is skipped."""

    def page(self, body):
        fields, slides = render_deck.parse_deck("# T\n---\n" + body)
        d = Path(tempfile.mkdtemp())
        page = d / "deck.html"
        page.write_text(render_deck.build_html(fields, slides, d, str(d), []), encoding="utf-8")
        return page

    def chrome(self):
        chrome = render_deck.find_chrome()
        if not chrome:
            self.skipTest("no Chrome or Chromium")
        return chrome

    def local_mermaid(self):
        path = os.environ.get("MERMAID_JS")
        if not path or not render_deck.mermaid_source([])[0].startswith("file:"):
            self.skipTest("MERMAID_JS does not name a verified copy of the pinned Mermaid")

    def test_a_diagram_is_drawn_and_a_broken_one_reported(self):
        chrome = self.chrome()
        self.local_mermaid()
        good = "## Flow\n```mermaid\nflowchart LR\n  A[Client] --> B[Server]\n```\n"
        bad = "---\n## Broken\n```mermaid\nflowchart LR\n  A --> -->\n```\n"
        found = render_deck.inspect(chrome, self.page(good + bad))
        self.assertEqual(found["missing"], [])
        self.assertEqual([n for n, _ in found["errors"]], [3])

    def test_without_the_library_the_diagram_is_reported_missing(self):
        chrome = self.chrome()
        old = os.environ.get("MERMAID_JS")
        os.environ["MERMAID_JS"] = "/nonexistent/mermaid.js"
        saved, render_deck.MERMAID_CDN = render_deck.MERMAID_CDN, "file:///nonexistent/mermaid.js"
        try:
            found = render_deck.inspect(chrome, self.page("## Flow\n```mermaid\nflowchart LR\n  A --> B\n```\n"))
        finally:
            render_deck.MERMAID_CDN = saved
            if old is None:
                del os.environ["MERMAID_JS"]
            else:
                os.environ["MERMAID_JS"] = old
        self.assertEqual(found["missing"], [2])


if __name__ == "__main__":
    unittest.main(verbosity=0)

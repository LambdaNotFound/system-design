#!/usr/bin/env python3
"""Render a deck written in make-deck's Markdown to a 16:9 PDF. Standard library only.

    render_deck.py deck.md                 writes deck.pdf beside it
    render_deck.py deck.md --out x.pdf     writes x.pdf
    render_deck.py deck.md --notes         also writes deck.notes.pdf: each slide's title and notes
    render_deck.py deck.md --html-only     writes deck.html and stops (no browser needed)

The deck format is in references/deck-spec.md. The PDF is printed by a local Chrome or
Chromium in headless mode: set CHROME_PATH to pick one, or the script looks in the usual
places. Before printing it loads the page once more to find slides whose content overflows
the page, and reports them; it prints the PDF either way, so the warning is the signal.

A mermaid block is drawn by Mermaid itself, in the same browser, at one pinned version.
The page loads it from a local copy when MERMAID_JS names one or the cache path below holds
one, and otherwise from the jsDelivr CDN, which the browser checks against the pinned hash.
A local copy is checked against the same hash before use. Offline, with no local copy, a
diagram prints as its source and the script says so. To work offline, fetch the copy once:

    mkdir -p ~/.cache/make-deck && curl -sL https://registry.npmjs.org/mermaid/-/mermaid-11.4.1.tgz \
      | tar -xzO package/dist/mermaid.min.js > ~/.cache/make-deck/mermaid-11.4.1.min.js

Each diagram is scaled to fill the space its slide leaves, up to 1.6 times its natural size;
one scaled below 0.6 is reported, because its text is then too small to read.

Exit codes: 0 rendered with no overflow; 1 rendered, but a slide overflows; 2 could not
render (no browser, bad input).
"""
import argparse
import base64
import hashlib
import html
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

PAGE_W, PAGE_H = "13.333in", "7.5in"   # 16:9

MERMAID_VERSION = "11.4.1"
MERMAID_SHA384 = "rbtjAdnIQE/aQJGEgXrVUlMibdfTSa4PQju4HDhN3sR2PmaKFzhEafuePsl9H/9I"
MERMAID_CDN = f"https://cdn.jsdelivr.net/npm/mermaid@{MERMAID_VERSION}/dist/mermaid.min.js"
MERMAID_CACHE = Path.home() / ".cache" / "make-deck" / f"mermaid-{MERMAID_VERSION}.min.js"

CSS = """
@page { size: %(w)s %(h)s; margin: 0; }
* { box-sizing: border-box; }
html, body { margin: 0; padding: 0; background: #fff; }
body { font-family: -apple-system, "Helvetica Neue", "Segoe UI", Arial, sans-serif;
       color: #1a1a1a; -webkit-print-color-adjust: exact; print-color-adjust: exact; }
.slide { width: %(w)s; height: %(h)s; padding: 0.6in 0.85in 0.75in; position: relative;
         overflow: hidden; page-break-after: always; break-after: page;
         display: flex; flex-direction: column; }
.slide:last-child { page-break-after: auto; break-after: auto; }
.body { flex: 1; min-height: 0; overflow: hidden; display: flex; flex-direction: column; }
/* Nothing but a figure shrinks: a shrunk code block clips itself, and the overflow check
   would not see it. */
.body > * { flex-shrink: 0; }
.body > figure, .body > .diagram { flex-shrink: 1; }
.diagram { flex: 1; min-height: 1.5in; display: flex; flex-direction: column; align-items: center;
           justify-content: center; }
.diagram > pre { flex-shrink: 0; align-self: stretch; }
.diagram svg { max-width: none !important; }
h1 { font-size: 46pt; font-weight: 650; margin: 0 0 0.25in; line-height: 1.15; }
h2 { font-size: 34pt; font-weight: 650; margin: 0 0 0.3in; padding-bottom: 0.12in;
     border-bottom: 2px solid #d0d0d0; line-height: 1.2; }
h3 { font-size: 25pt; font-weight: 600; margin: 0.1in 0 0.08in; color: #333; }
p, li { font-size: 25pt; line-height: 1.35; }
p { margin: 0 0 0.14in; }
ul, ol { margin: 0 0 0.14in; padding-left: 0.4in; }
li { margin: 0 0 0.12in; }
li li { font-size: 21pt; color: #333; }
li ul, li ol { margin: 0.06in 0 0; }
code { font-family: "SF Mono", Menlo, Consolas, "DejaVu Sans Mono", monospace; font-size: 0.88em;
       background: #f2f2f2; padding: 0.02in 0.06in; border-radius: 3px; }
pre { background: #f6f6f6; border: 1px solid #e2e2e2; border-radius: 4px; padding: 0.14in 0.2in;
      margin: 0 0 0.14in; overflow: hidden; }
pre code { background: none; padding: 0; font-size: 16pt; line-height: 1.35; white-space: pre; }
table { border-collapse: collapse; margin: 0 0 0.2in; font-size: 20pt; }
th, td { border-bottom: 1px solid #dcdcdc; padding: 0.1in 0.3in 0.1in 0; text-align: left;
         vertical-align: top; }
th { font-weight: 650; border-bottom: 2px solid #999; }
img, svg.diagram { max-width: 100%%; max-height: 4.6in; object-fit: contain; align-self: center; }
figure { margin: 0; display: flex; flex-direction: column; min-height: 0; flex: 1; }
figcaption { font-size: 12pt; color: #666; margin-top: 0.06in; text-align: center; }
.caption { font-size: 12pt; color: #777; }
.footer { position: absolute; left: 0.85in; right: 0.85in; bottom: 0.3in; font-size: 12pt;
          color: #888; display: flex; justify-content: space-between; }
.title-slide { justify-content: center; }
.title-slide .sub { font-size: 26pt; color: #444; margin: 0 0 0.1in; }
.title-slide .meta { font-size: 17pt; color: #777; }
.notes-page h2 { font-size: 22pt; }
.notes-page p, .notes-page li { font-size: 18pt; }
""" % {"w": PAGE_W, "h": PAGE_H}

# Draws the Mermaid diagrams, fits each to its slot, then marks every slide whose content
# does not fit or whose diagram is too small to read, so --dump-dom can report them.
PAGE_JS = """
<script>
async function drawDiagrams() {
  const slots = Array.from(document.querySelectorAll('.diagram'));
  if (!slots.length) return;
  if (!window.mermaid) {
    slots.forEach(d => { d.setAttribute('data-mermaid', 'missing');
      d.insertAdjacentHTML('beforeend', '<p class="caption">Mermaid source: the library did not load</p>'); });
    return;
  }
  mermaid.initialize({ startOnLoad: false, theme: 'neutral', securityLevel: 'strict',
    fontFamily: getComputedStyle(document.body).fontFamily,
    themeVariables: { fontSize: '20px', xyChart: { plotColorPalette: '#7a8594' } },
    flowchart: { wrappingWidth: 320, nodeSpacing: 40, rankSpacing: 55, padding: 12 },
    sequence: { actorFontSize: 20, messageFontSize: 20, noteFontSize: 18, wrap: false },
    er: { fontSize: 18 },
    state: { fontSize: 20 },
    timeline: { padding: 12 },
    xyChart: { width: 1000, height: 430, titleFontSize: 22, showTitle: true,
      xAxis: { labelFontSize: 20, titleFontSize: 20 }, yAxis: { labelFontSize: 18, titleFontSize: 18 } } });
  for (let i = 0; i < slots.length; i++) {
    const slot = slots[i];
    const src = slot.querySelector('code').textContent;
    try {
      const { svg } = await mermaid.render('mmd' + i, src);
      slot.innerHTML = svg;
      const el = slot.querySelector('svg');
      const vb = el.viewBox && el.viewBox.baseVal;
      const w = vb && vb.width ? vb.width : el.getBBox().width;
      const h = vb && vb.height ? vb.height : el.getBBox().height;
      const k = Math.min(slot.clientWidth / w, slot.clientHeight / h, 1.6);
      el.setAttribute('width', w * k); el.setAttribute('height', h * k);
      el.style.width = (w * k) + 'px'; el.style.height = (h * k) + 'px';
      slot.setAttribute('data-mermaid', 'ok');
      slot.setAttribute('data-scale', k.toFixed(2));
    } catch (e) {
      slot.setAttribute('data-mermaid', 'error');
      slot.setAttribute('data-error', String(e && e.message || e).slice(0, 160));
      document.querySelectorAll('[id^="dmmd' + i + '"]').forEach(n => n.remove());
    }
  }
}
window.addEventListener('load', async () => {
  try { await drawDiagrams(); } catch (e) {}
  document.querySelectorAll('.slide').forEach((s) => {
    const b = s.querySelector('.body') || s;
    if (b.scrollHeight > b.clientHeight + 2 || b.scrollWidth > b.clientWidth + 2) {
      s.setAttribute('data-overflow', 'yes');
    }
  });
  document.body.setAttribute('data-checked', 'yes');
});
</script>
"""


def mermaid_source(warnings):
    """Return (src, integrity) for the script tag: a verified local copy, else the CDN."""
    local = os.environ.get("MERMAID_JS")
    path = Path(local).expanduser() if local else MERMAID_CACHE
    if path.exists():
        digest = base64.b64encode(hashlib.sha384(path.read_bytes()).digest()).decode()
        if digest == MERMAID_SHA384:
            return path.resolve().as_uri(), None
        warnings.append(f"{path} is not Mermaid {MERMAID_VERSION} (hash mismatch); using the CDN")
    elif local:
        warnings.append(f"MERMAID_JS names no file: {local}; using the CDN")
    return MERMAID_CDN, "sha384-" + MERMAID_SHA384


CHROME_CANDIDATES = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "chrome",
    "/opt/pw-browsers/chromium/chrome-linux/chrome",
]


# ---------------------------------------------------------------- parsing

def split_front_matter(text):
    """Return (fields, rest). Front matter is a leading block between two `---` lines."""
    fields = {}
    if text.startswith("---\n"):
        end = text.find("\n---\n", 4)
        if end != -1:
            for line in text[4:end].splitlines():
                m = re.match(r"^([A-Za-z_][\w-]*):\s*(.*)$", line)
                if m:
                    fields[m.group(1)] = m.group(2).strip().strip("\"'")
            return fields, text[end + 5:]
    return fields, text


def split_slides(text):
    """Split on lines that are exactly `---`, outside fenced code."""
    slides, current, fenced = [], [], False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
        if not fenced and line.strip() == "---":
            slides.append("\n".join(current))
            current = []
        else:
            current.append(line)
    slides.append("\n".join(current))
    return [s for s in slides if s.strip()]


def split_notes(slide):
    """A line `Notes:` starts the speaker notes, which run to the end of the slide."""
    lines = slide.splitlines()
    fenced = False
    for i, line in enumerate(lines):
        if line.lstrip().startswith("```"):
            fenced = not fenced
        if not fenced and re.match(r"^Notes:\s*", line):
            first = re.sub(r"^Notes:\s*", "", line)
            return "\n".join(lines[:i]), "\n".join([first] + lines[i + 1:]).strip()
    return slide, ""


def parse_deck(text):
    fields, rest = split_front_matter(text)
    slides = []
    for raw in split_slides(rest):
        body, notes = split_notes(raw)
        slides.append({"body": body.strip("\n"), "notes": notes})
    return fields, slides


def slide_title(body):
    for line in body.splitlines():
        m = re.match(r"^(#{1,2})\s+(.*)$", line)
        if m:
            return m.group(2).strip()
    return ""


# ---------------------------------------------------------------- Markdown subset to HTML

def inline(text):
    """Escape, then apply `code`, **bold**, *em* and [text](url) links (rendered as text).
    Code spans are set aside first, so bold or emphasis may wrap them."""
    spans = []

    def keep(m):
        spans.append("<code>" + html.escape(m.group(1)) + "</code>")
        return "\x00%d\x00" % (len(spans) - 1)

    s = re.sub(r"`([^`]+)`", keep, text)
    s = html.escape(s)
    s = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", r"\1", s)
    s = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?![\w*])", r"<em>\1</em>", s)
    return re.sub(r"\x00(\d+)\x00", lambda m: spans[int(m.group(1))], s)


def render_list(lines):
    """Two levels of `-`, `*` or `1.` items; an indent of two or more spaces nests."""
    html_out, stack = [], []   # stack of (indent, tag)
    for line in lines:
        m = re.match(r"^(\s*)([-*]|\d+\.)\s+(.*)$", line)
        if not m:
            if html_out:   # a continuation line joins the previous item
                html_out[-1] = html_out[-1][:-5] + " " + inline(line.strip()) + "</li>"
            continue
        indent, marker, item = len(m.group(1).expandtabs(2)), m.group(2), m.group(3)
        tag = "ol" if marker[0].isdigit() else "ul"
        while stack and indent < stack[-1][0]:
            entry = stack.pop()
            html_out.append(f"</{entry[1]}>" + ("</li>" if entry[2] else ""))
        if not stack or indent > stack[-1][0]:
            if stack and html_out and html_out[-1].endswith("</li>"):
                html_out[-1] = html_out[-1][:-5]   # open the nested list inside the last item
                stack.append((indent, tag, True))
            else:
                stack.append((indent, tag, False))
            html_out.append(f"<{tag}>")
        html_out.append(f"<li>{inline(item)}</li>")
    while stack:
        entry = stack.pop()
        html_out.append(f"</{entry[1]}>" + ("</li>" if entry[2] else ""))
    return "\n".join(html_out)


def render_table(lines):
    rows = [[c.strip() for c in l.strip().strip("|").split("|")] for l in lines]
    rows = [r for r in rows if not all(re.fullmatch(r":?-{2,}:?", c) for c in r)]
    if not rows:
        return ""
    head, body = rows[0], rows[1:]
    out = ["<table><thead><tr>"] + [f"<th>{inline(c)}</th>" for c in head] + ["</tr></thead><tbody>"]
    for r in body:
        out += ["<tr>"] + [f"<td>{inline(c)}</td>" for c in r] + ["</tr>"]
    out.append("</tbody></table>")
    return "".join(out)


def render_mermaid(source, workdir, index, warnings):
    """A slot the page script replaces with the drawn diagram; until then, the source."""
    return ('<div class="diagram"><pre><code>' + html.escape(source) + "</code></pre></div>")


def render_body(body, base, workdir, warnings, counter):
    lines = body.splitlines()
    out, i = [], 0
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        fence = re.match(r"^\s*```\s*([\w-]*)", line)
        if fence:
            lang, block = fence.group(1), []
            i += 1
            while i < len(lines) and not lines[i].lstrip().startswith("```"):
                block.append(lines[i])
                i += 1
            i += 1
            source = "\n".join(block)
            if lang == "mermaid":
                counter[0] += 1
                out.append(render_mermaid(source, workdir, counter[0], warnings))
            else:
                out.append("<pre><code>" + html.escape(source) + "</code></pre>")
            continue
        heading = re.match(r"^(#{1,3})\s+(.*)$", line)
        if heading:
            level = len(heading.group(1))
            out.append(f"<h{level}>{inline(heading.group(2))}</h{level}>")
            i += 1
            continue
        image = re.match(r"^!\[([^\]]*)\]\(([^)\s]+)\)\s*$", line.strip())
        if image:
            alt, target = image.group(1), image.group(2)
            path = (base / target).resolve()
            if not path.exists():
                warnings.append(f"image not found: {target}")
            cap = f"<figcaption>{inline(alt)}</figcaption>" if alt else ""
            out.append(f'<figure><img src="{path.as_uri()}" alt="{html.escape(alt)}">{cap}</figure>')
            i += 1
            continue
        if line.lstrip().startswith("|"):
            block = []
            while i < len(lines) and lines[i].lstrip().startswith("|"):
                block.append(lines[i])
                i += 1
            out.append(render_table(block))
            continue
        if re.match(r"^\s*([-*]|\d+\.)\s+", line):
            block = []
            while i < len(lines) and lines[i].strip() and not lines[i].lstrip().startswith(("```", "|", "#")):
                block.append(lines[i])
                i += 1
            out.append(render_list(block))
            continue
        para = []
        while i < len(lines) and lines[i].strip() and not re.match(r"^\s*(```|#|\||[-*]\s|\d+\.\s|!\[)", lines[i]):
            para.append(lines[i].strip())
            i += 1
        if para:
            out.append(f"<p>{inline(' '.join(para))}</p>")
        else:
            i += 1
    return "\n".join(out)


def build_html(fields, slides, base, workdir, warnings, notes=False):
    title = fields.get("title", "") or (slide_title(slides[0]["body"]) if slides else "")
    total = len(slides)
    pages = []
    counter = [0]
    for n, slide in enumerate(slides, 1):
        body = slide["body"]
        first = n == 1 and re.match(r"^\s*#\s", body or "")
        footer = "" if first else (f'<div class="footer"><span>{html.escape(title)}</span>'
                                   f"<span>{n} / {total}</span></div>")
        if notes:
            content = (f"<h2>{inline(slide_title(body) or 'Slide %d' % n)}</h2>"
                       + render_body(slide["notes"] or "(no notes)", base, workdir, warnings, counter))
            pages.append(f'<section class="slide notes-page"><div class="body">{content}</div>'
                         f'<div class="footer"><span>Notes</span><span>{n} / {total}</span></div></section>')
            continue
        if first:
            lines = body.splitlines()
            head = [l for l in lines if re.match(r"^\s*#\s", l)][0]
            rest = "\n".join(l for l in lines if l is not head)
            sub = "".join(f'<p class="sub">{inline(l.strip())}</p>' for l in rest.splitlines() if l.strip())
            meta = " · ".join(html.escape(fields[k]) for k in ("author", "date") if fields.get(k))
            pages.append(f'<section class="slide title-slide"><h1>{inline(head.lstrip("# ").strip())}</h1>'
                         f'{sub}<p class="meta">{meta}</p></section>')
            continue
        content = render_body(body, base, workdir, warnings, counter)
        pages.append(f'<section class="slide"><div class="body">{content}</div>{footer}</section>')
    script = ""
    if counter[0]:
        src, integrity = mermaid_source(warnings)
        check = f' integrity="{integrity}" crossorigin="anonymous"' if integrity else ""
        script = f'<script src="{html.escape(src)}"{check}></script>'
    return ("<!doctype html><html><head><meta charset='utf-8'>"
            f"<title>{html.escape(title)}</title><style>{CSS}</style></head><body>"
            + "\n".join(pages) + script + PAGE_JS + "</body></html>")


# ---------------------------------------------------------------- browser

def find_chrome():
    env = os.environ.get("CHROME_PATH")
    if env:
        return env if (os.path.exists(env) or shutil.which(env)) else None
    for c in CHROME_CANDIDATES:
        if os.path.isabs(c) and os.path.exists(c):
            return c
        found = shutil.which(c)
        if found:
            return found
    for p in sorted(Path("/opt/pw-browsers").glob("chromium*/chrome-linux*/chrome")):
        return str(p)
    return None


def chrome_args(chrome):
    args = [chrome, "--headless=new", "--disable-gpu", "--hide-scrollbars",
            "--allow-file-access-from-files", "--no-first-run", "--no-default-browser-check"]
    if sys.platform.startswith("linux") and hasattr(os, "geteuid") and os.geteuid() == 0:
        args.append("--no-sandbox")   # Chrome refuses to run as root with its sandbox on
    return args


def inspect(chrome, html_path):
    """Load the page once and report what its script found, or None if it did not run."""
    done = subprocess.run(chrome_args(chrome) + ["--virtual-time-budget=15000", "--dump-dom",
                                                 html_path.as_uri()],
                          stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, universal_newlines=True,
                          timeout=180)
    dom = done.stdout
    if 'data-checked="yes"' not in dom:
        return None
    found = {"overflow": [], "small": [], "missing": [], "errors": []}
    parts = re.split(r'(<section class="slide[^"]*"[^>]*>)', dom)
    n = 0
    for i in range(1, len(parts), 2):
        n += 1
        tag, inner = parts[i], parts[i + 1] if i + 1 < len(parts) else ""
        if 'data-overflow="yes"' in tag:
            found["overflow"].append(n)
        for attrs in re.findall(r'<div class="diagram"([^>]*)>', inner):
            state = re.search(r'data-mermaid="(\w+)"', attrs)
            state = state.group(1) if state else "missing"
            if state == "missing":
                found["missing"].append(n)
            elif state == "error":
                err = re.search(r'data-error="([^"]*)"', attrs)
                found["errors"].append((n, html.unescape(err.group(1)) if err else "unknown error"))
            else:
                scale = re.search(r'data-scale="([\d.]+)"', attrs)
                if scale and float(scale.group(1)) < 0.6:
                    found["small"].append((n, float(scale.group(1))))
    return found


def overflowing(chrome, html_path):
    found = inspect(chrome, html_path)
    return None if found is None else found["overflow"]


def print_pdf(chrome, html_path, pdf_path):
    done = subprocess.run(chrome_args(chrome) + ["--no-pdf-header-footer", "--virtual-time-budget=15000",
                                                 f"--print-to-pdf={pdf_path}", html_path.as_uri()],
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True,
                          timeout=180)
    return pdf_path.exists() and pdf_path.stat().st_size > 0, done.stdout


def main():
    ap = argparse.ArgumentParser(description="Render a make-deck Markdown deck to PDF.")
    ap.add_argument("deck")
    ap.add_argument("--out")
    ap.add_argument("--notes", action="store_true")
    ap.add_argument("--html-only", action="store_true")
    a = ap.parse_args()

    deck = Path(a.deck).resolve()
    if not deck.exists():
        print(f"render_deck: no such file: {a.deck}")
        return 2
    fields, slides = parse_deck(deck.read_text(encoding="utf-8"))
    if not slides:
        print("render_deck: the deck has no slides")
        return 2
    out = Path(a.out).resolve() if a.out else deck.with_suffix(".pdf")
    warnings = []
    workdir = tempfile.mkdtemp(prefix="deck-")
    html_path = Path(workdir) / "deck.html"
    html_path.write_text(build_html(fields, slides, deck.parent, workdir, warnings), encoding="utf-8")
    if a.html_only:
        target = deck.with_suffix(".html")
        shutil.copyfile(html_path, target)
        print(f"render_deck: wrote {target.name}, {len(slides)} slides")
        for w in warnings:
            print(f"  warn   {w}")
        return 0

    chrome = find_chrome()
    if not chrome:
        print("render_deck: no Chrome or Chromium found. Install one, or set CHROME_PATH to its binary.")
        return 2
    found = inspect(chrome, html_path)
    ok, log = print_pdf(chrome, html_path, out)
    if not ok:
        print("render_deck: the browser wrote no PDF:\n" + log.strip()[-600:])
        return 2
    if a.notes:
        notes_html = Path(workdir) / "notes.html"
        notes_html.write_text(build_html(fields, slides, deck.parent, workdir, [], notes=True), encoding="utf-8")
        print_pdf(chrome, notes_html, out.with_suffix(".notes.pdf"))

    print(f"render_deck: wrote {out.name}, {len(slides)} slides"
          + (f", and {out.with_suffix('.notes.pdf').name}" if a.notes else ""))
    for w in sorted(set(warnings)):
        print(f"  warn   {w}")
    if found is None:
        print("  warn   the page checks did not run; look at every slide in the PDF")
        return 0
    status = 0
    if found["missing"]:
        slides_ = ", ".join(map(str, sorted(set(found["missing"]))))
        print(f"  warn   Mermaid did not load, so the diagrams on slide {slides_} printed as source:"
              " go online, or set MERMAID_JS (see the script's help)")
    for n, err in found["errors"]:
        print(f"  error  slide {n}: the Mermaid diagram does not parse: {err}")
        status = 1
    for n, k in found["small"]:
        print(f"  warn   slide {n}: the diagram is drawn at {k:.2f} of its size, too small to read:"
              " cut nodes, shorten labels, or give it the slide alone")
    if found["overflow"]:
        print(f"  error  content overflows the page on slide {', '.join(map(str, found['overflow']))}:"
              " cut it or split the slide")
        status = 1
    return status


if __name__ == "__main__":
    sys.exit(main())

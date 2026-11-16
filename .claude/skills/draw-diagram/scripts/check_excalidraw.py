#!/usr/bin/env python3
"""Check an .excalidraw file for the defects Excalidraw opens without complaint.

Usage:
    python3 check_excalidraw.py <file.excalidraw>

Errors (exit 1): the file is not Excalidraw JSON, an id repeats, a binding is
listed on one end only, or text bound to a shape sits outside it or is too tall.
Warnings (exit 0): text wider than its shape leaves or off its shape's center,
an arrow label with too little arrow under it or on a sharp corner, by Excalidraw's
own font widths, and layout read from the coordinates: shapes that overlap without
nesting, arrow ends off their shapes, arrows through shapes or text, text over text
or across a shape's outline, step badges off their spot, and boundary children too
close to the boundary label. Exit 2: unreadable file.

Stdlib only. It draws nothing: it follows a curved arrow along its points, and it
does not judge arrow crossings, sizes, or colors.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import unicodedata
from pathlib import Path

# Advance widths of ASCII 32..126 in 1/1000 em, measured in Chrome 2026-11-13 from the font
# files Excalidraw 0.18.0 loads. Liberation Sans matches Arial; Cascadia Code is 586 for all.
SANS = (
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722,
    722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722,
    667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556,
    556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500,
    500, 334, 260, 334, 584
)
HAND = (
    400, 314, 371, 783, 721, 928, 718, 218, 441, 402, 525, 550, 257, 411, 274, 561, 664, 427,
    700, 608, 585, 618, 640, 558, 636, 629, 264, 298, 550, 550, 550, 466, 829, 676, 761, 629,
    780, 707, 661, 780, 573, 545, 569, 613, 543, 766, 632, 767, 698, 768, 736, 622, 857, 730,
    592, 786, 628, 564, 832, 472, 589, 497, 510, 670, 600, 576, 555, 504, 605, 537, 497, 555,
    567, 244, 328, 533, 225, 663, 526, 600, 537, 539, 412, 543, 553, 548, 525, 693, 591, 530,
    572, 504, 299, 544, 669
)
MONO = 586
FALLBACK = 600  # references/excalidraw.md's 0.6 x fontSize rule, for other fonts and characters
WIDE = 1000
WIDTHS = {2: SANS, 9: SANS, 5: HAND}

# The spacing rules of references/excalidraw.md.
BOX_PADDING = 40
ARROW_SLACK = 60
BADGE_ROOM = 100  # an arrow that also carries a step badge needs this much more than its label
BADGE_OFFSET = 24
LABEL_ROOM = 44
TEXT_CONTAINERS = ("rectangle", "ellipse", "diamond", "arrow")
POINT_TYPES = ("arrow", "line", "freedraw")

# Layout tolerances, in px.
END_SLACK = 12     # an arrow end may sit this far from its shape's edge: the 4 px gap plus 8
EDGE_MARGIN = 3    # a path or shape this close inside an edge still only touches it
BADGE_MAX = 48     # a step badge is an ellipse with a label, at most this wide and tall
BADGE_SLACK = 8
MARK_MAX = 16      # a shape this small, centered on an arrow, is a dot on that arrow
CENTER_SLACK = 6   # text bound to a shape may sit this far off the shape's center
LABEL_BAND = 30    # free text within this far of a shape's top is that shape's label
SAMPLE_STEP = 2

Box = tuple[float, float, float, float]
Point = tuple[float, float]


def _as_num(value, default=0):
    return value if isinstance(value, (int, float)) and not isinstance(value, bool) else default


def _num(el: dict, key: str, default=0):
    return _as_num(el.get(key, default), default)


def _lines(el: dict) -> list[str]:
    return str(el.get("text", "")).split("\n")


def _char_width(ch: str, family) -> int:
    if unicodedata.east_asian_width(ch) in ("W", "F"):
        return WIDE
    if family == 3:
        return MONO
    table = WIDTHS.get(family)
    return table[ord(ch) - 32] if table and 32 <= ord(ch) < 127 else FALLBACK


def text_width(el: dict) -> float:
    family = el.get("fontFamily", 9)
    family = family if isinstance(family, int) and not isinstance(family, bool) else None
    longest = max(sum(_char_width(ch, family) for ch in line) for line in _lines(el))
    return longest * _num(el, "fontSize", 20) / 1000


def needed_height(el: dict) -> float:
    return _num(el, "fontSize", 20) * _num(el, "lineHeight", 1.25) * len(_lines(el))


def _room_share(shape: dict, text_height: float) -> float:
    """The share of a diamond's or ellipse's width that a centered text block of text_height can use."""
    height = _num(shape, "height")
    ratio = min(text_height / height, 1) if height > 0 else 1
    return 1 - ratio if shape.get("type") == "diamond" else math.sqrt(1 - ratio * ratio)


def usable_width(shape: dict, text_height: float) -> float:
    """Width a centered text block of text_height can use inside shape."""
    if shape.get("type") == "rectangle":
        return _num(shape, "width") - BOX_PADDING
    return _num(shape, "width") * _room_share(shape, text_height)


def required_width(shape: dict, need: float, text_height: float) -> float | None:
    """The shape width that leaves need px for the text, or None if no width does."""
    if shape.get("type") == "rectangle":
        return need + BOX_PADDING
    share = _room_share(shape, text_height)
    return need / share if share > 0 else None


def _text_left(el: dict, width: float) -> float:
    """Where a text line of width starts: Excalidraw aligns it inside the element's own width."""
    x, declared = _num(el, "x"), _num(el, "width")
    align = el.get("textAlign", "left")
    return x + (declared - width) / 2 if align == "center" else x + declared - width if align == "right" else x


def _free_text_box(el: dict) -> tuple[float, float, float, float]:
    w = text_width(el)
    x0, y = _text_left(el, w), _num(el, "y")
    return x0, y, x0 + w, y + needed_height(el)


def _off_center(el: dict, container: dict, free: list[tuple[float, float, float, float]]) -> str | None:
    """A warning when text bound to a shape does not sit at the shape's center. A shape that also
    holds free text, such as a second line or a body under a title, is laid out by hand: no warning."""
    cx0, cy0 = _num(container, "x"), _num(container, "y")
    cx1, cy1 = cx0 + _num(container, "width"), cy0 + _num(container, "height")
    if any(cx0 <= b[0] and cy0 <= b[1] and b[2] <= cx1 and b[3] <= cy1 for b in free):
        return None
    w, h = text_width(el), needed_height(el)
    dx = _text_left(el, w) + w / 2 - (cx0 + cx1) / 2
    dy = _num(el, "y") + h / 2 - (cy0 + cy1) / 2
    if max(abs(dx), abs(dy)) <= CENTER_SLACK:
        return None
    align = "" if el.get("textAlign") == "center" else ', or set textAlign to "center"'
    return (f"'{el['id']}' sits {math.hypot(dx, dy):.0f} px off the center of '{container['id']}'; "
            f"set x = {_num(el, 'x') - dx:.0f}, y = {_num(el, 'y') - dy:.0f}{align}")


def _path(el: dict) -> list[Point]:
    points = el.get("points")
    if not isinstance(points, list):
        return []
    x, y = _num(el, "x"), _num(el, "y")
    return [(x + _as_num(p[0]), y + _as_num(p[1])) for p in points if isinstance(p, list) and len(p) >= 2]


def _ref(el: dict, key: str) -> str | None:
    value = el.get(key)
    return value if isinstance(value, str) and value else None


def arrow_length(el: dict) -> float:
    path = _path(el)
    return sum(math.dist(a, b) for a, b in zip(path, path[1:]))


def _outgoing_refs(el: dict) -> list[tuple[str, str]]:
    refs = [("containerId", _ref(el, "containerId"))]
    for key in ("startBinding", "endBinding"):
        binding = el.get(key)
        refs.append((key, _ref(binding, "elementId") if isinstance(binding, dict) else None))
    return [(key, target) for key, target in refs if target]


def _bound_items(el: dict) -> list[dict]:
    items = el.get("boundElements")
    return [b for b in items if isinstance(b, dict)] if isinstance(items, list) else []


def _bound_ids(el: dict) -> list:
    return [_ref(b, "id") for b in _bound_items(el)]


def _center_inside(el: dict, box: dict) -> bool:
    cx = _num(el, "x") + _num(el, "width") / 2
    cy = _num(el, "y") + _num(el, "height") / 2
    return (_num(box, "x") <= cx <= _num(box, "x") + _num(box, "width")
            and _num(box, "y") <= cy <= _num(box, "y") + _num(box, "height"))


FIELD_TYPES = (("groupIds", list, "a list of ids"), ("boundElements", list, "a list"),
               ("points", list, "a list of [x, y] pairs"), ("startBinding", dict, "an object"),
               ("endBinding", dict, "an object"), ("containerId", str, "an id"), ("frameId", str, "an id"),
               ("text", str, "a string"), ("fontFamily", int, "a number"))


def index(elements: list) -> tuple[dict[str, dict], list[str]]:
    errors: list[str] = []
    by_id: dict[str, dict] = {}
    for i, el in enumerate(elements):
        if not isinstance(el, dict):
            errors.append(f"elements[{i}] is not an object")
        elif not isinstance(el.get("id"), str) or not el["id"]:
            errors.append(f"elements[{i}] has no string 'id'")
        elif el["id"] in by_id:
            errors.append(f"duplicate id '{el['id']}'")
        else:
            by_id[el["id"]] = el
            bad = [k for k in ("x", "y", "width", "height", "fontSize")
                   if k in el and _num(el, k, None) is None]
            if bad:
                errors.append(f"'{el['id']}' has non-numeric {', '.join(bad)}")
            wrong = [f"{key} (expected {what})" for key, kind, what in FIELD_TYPES
                     if el.get(key) is not None and (not isinstance(el[key], kind) or isinstance(el[key], bool))]
            wrong += [f"{key}.elementId (expected an id)" for key in ("startBinding", "endBinding")
                      if isinstance(el.get(key), dict) and not _ref(el[key], "elementId")]
            if wrong:
                errors.append(f"'{el['id']}' has a bad {', '.join(wrong)}")
    return by_id, errors


def check_elements(by_id: dict[str, dict]) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    free = [_free_text_box(el) for el in by_id.values()
            if el.get("type") == "text" and not el.get("isDeleted") and not _ref(el, "containerId")]
    for el_id, el in by_id.items():
        if el.get("isDeleted"):
            continue
        for field, target in _outgoing_refs(el):
            if target not in by_id:
                errors.append(f"'{el_id}'.{field} -> missing id '{target}'")
            elif el_id not in _bound_ids(by_id[target]):
                errors.append(f"'{el_id}'.{field} -> '{target}', but '{target}'.boundElements does not list '{el_id}'")
        for member in _bound_ids(el):
            if member not in by_id:
                errors.append(f"'{el_id}'.boundElements lists missing id '{member}'")
            elif el_id not in [target for _, target in _outgoing_refs(by_id[member])]:
                errors.append(f"'{el_id}'.boundElements lists '{member}', which does not point back to '{el_id}'")
        frame_id = _ref(el, "frameId")
        if frame_id and frame_id not in by_id:
            errors.append(f"'{el_id}'.frameId -> missing id '{frame_id}'")

        container = by_id.get(_ref(el, "containerId") or "")
        if container is None or el.get("type") != "text":
            continue
        kind = container.get("type")
        if kind not in TEXT_CONTAINERS:
            errors.append(f"'{el_id}' is bound to '{container['id']}', a {kind}; only rectangle, ellipse, diamond, and arrow can hold text")
            continue
        if kind == "arrow":
            continue  # Excalidraw places arrow labels itself; check_layout checks the room under them.
        # Excalidraw draws text in a shape at the text's own x/y, from its top edge,
        # so a text box as tall as its shape puts the words at the top.
        if not _center_inside(el, container):
            errors.append(f"'{el_id}' is bound to '{container['id']}' but its center is outside that shape; center it")
        elif _num(el, "height") > 1.5 * needed_height(el):
            errors.append(f"'{el_id}' is {_num(el, 'height'):.0f} px tall, but its text needs {needed_height(el):.0f} px, so it renders at the top of its shape; use height = fontSize x 1.25 x lines")
        elif _off_center(el, container, free):
            warnings.append(_off_center(el, container, free))
        need, room = text_width(el), usable_width(container, needed_height(el))
        if need > room:
            wide = required_width(container, need, needed_height(el))
            fix = (f"widen '{container['id']}' by {math.ceil(wide - _num(container, 'width'))} px, or break the line" if wide
                   else f"make '{container['id']}' taller, or break the line")
            warnings.append(f"'{el_id}' needs about {need:.0f} px, but {kind} '{container['id']}' leaves {max(room, 0):.0f} px; {fix}")
    return errors, warnings


# Layout: geometry read from the coordinates.

def _area(box: Box) -> float:
    return (box[2] - box[0]) * (box[3] - box[1])


def _within(inner: Box, outer: Box, slack: float = 1) -> bool:
    return (outer[0] - slack <= inner[0] and outer[1] - slack <= inner[1]
            and inner[2] <= outer[2] + slack and inner[3] <= outer[3] + slack)


def _holds(outer: Box, inner: Box) -> bool:
    return _within(inner, outer) and _area(outer) > _area(inner)


def _overlap(a: Box, b: Box) -> float:
    """How deep two boxes overlap, in px; 0 or less means apart or touching."""
    return min(min(a[2], b[2]) - max(a[0], b[0]), min(a[3], b[3]) - max(a[1], b[1]))


def _in_box(box: Box, p: Point, margin: float = 0) -> bool:
    return box[0] + margin < p[0] < box[2] - margin and box[1] + margin < p[1] < box[3] - margin


def _nearest_on_segment(a: Point, b: Point, p: Point) -> tuple[Point, float]:
    """The point on segment a-b nearest p, and how far along the segment it is."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    length2 = dx * dx + dy * dy
    t = 0 if length2 == 0 else max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length2))
    return (a[0] + t * dx, a[1] + t * dy), t * math.sqrt(length2)


def _distance_along(path: list[Point], p: Point) -> tuple[float, float]:
    """Distance from p to the path, and the path length from the start to the nearest point."""
    best, along, walked = math.inf, 0.0, 0.0
    for a, b in zip(path, path[1:]):
        near, offset = _nearest_on_segment(a, b, p)
        if math.dist(near, p) < best:
            best, along = math.dist(near, p), walked + offset
        walked += math.dist(a, b)
    return best, along


def _samples(path: list[Point]) -> list[Point]:
    points = path[:1]
    for a, b in zip(path, path[1:]):
        steps = max(1, math.ceil(math.dist(a, b) / SAMPLE_STEP))
        points += [(a[0] + (b[0] - a[0]) * i / steps, a[1] + (b[1] - a[1]) * i / steps) for i in range(1, steps + 1)]
    return points


def _label_point(path: list[Point]) -> Point:
    """Where Excalidraw centers a bound arrow label: the middle point, or the middle segment's midpoint."""
    if len(path) % 2 == 1:
        return path[len(path) // 2]
    a, b = path[len(path) // 2 - 1], path[len(path) // 2]
    return (a[0] + b[0]) / 2, (a[1] + b[1]) / 2


class Shape:
    """A closed outline: a rectangle, an ellipse, or a polygon (diamond or closed line)."""

    def __init__(self, el_id: str, kind: str, box: Box, poly: list[Point] | None = None,
                 group: str | None = None, badge: bool = False):
        self.id, self.kind, self.box, self.poly, self.group, self.badge = el_id, kind, box, poly, group, badge
        self._outline: list[Point] | None = None

    def outline(self) -> list[Point]:
        """Points every 2 px or so along the shape's edge."""
        if self._outline is None:
            x0, y0, x1, y1 = self.box
            if self.kind == "rectangle":
                self._outline = _samples([(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)])
            elif self.kind == "ellipse":
                cx, cy, a, b = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
                n = max(24, math.ceil(math.pi * (a + b) / SAMPLE_STEP))
                self._outline = [(cx + a * math.cos(2 * math.pi * i / n), cy + b * math.sin(2 * math.pi * i / n)) for i in range(n)]
            else:
                self._outline = _samples(self.poly)
        return self._outline

    def inside(self, p: Point, margin: float = 0) -> bool:
        x0, y0, x1, y1 = self.box
        if self.kind == "rectangle":
            return _in_box(self.box, p, margin)
        if self.kind == "ellipse":
            a, b = (x1 - x0) / 2 - margin, (y1 - y0) / 2 - margin
            if a <= 0 or b <= 0:
                return False
            return ((p[0] - (x0 + x1) / 2) / a) ** 2 + ((p[1] - (y0 + y1) / 2) / b) ** 2 < 1
        crossings = sum(1 for (ax, ay), (bx, by) in zip(self.poly, self.poly[1:])
                        if (ay > p[1]) != (by > p[1]) and p[0] < ax + (p[1] - ay) * (bx - ax) / (by - ay))
        return crossings % 2 == 1 and math.dist(self.nearest_edge(p), p) > margin

    def nearest_edge(self, p: Point) -> Point:
        x0, y0, x1, y1 = self.box
        if self.kind == "rectangle":
            if not _in_box(self.box, p):
                return min(max(p[0], x0), x1), min(max(p[1], y0), y1)
            return min(((x0, p[1]), (x1, p[1]), (p[0], y0), (p[0], y1)), key=lambda q: math.dist(q, p))
        if self.kind == "ellipse":
            # The edge point on the ray from the center: exact for circles, close for the rest.
            cx, cy, a, b = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
            r = math.hypot((p[0] - cx) / a, (p[1] - cy) / b) if a > 0 and b > 0 else 0
            return (cx + (p[0] - cx) / r, cy + (p[1] - cy) / r) if r else (x1, cy)
        nearest = [_nearest_on_segment(a, b, p)[0] for a, b in zip(self.poly, self.poly[1:])]
        return min(nearest, key=lambda q: math.dist(q, p))


def _group(el: dict) -> str | None:
    groups = el.get("groupIds")
    return groups[0] if isinstance(groups, list) and groups and isinstance(groups[0], str) else None


def _visible(el: dict) -> bool:
    return not (el.get("strokeColor") == "transparent"
                and el.get("backgroundColor", "transparent") == "transparent")


def shapes_of(by_id: dict[str, dict]) -> dict[str, Shape]:
    shapes: dict[str, Shape] = {}
    for el_id, el in by_id.items():
        kind = el.get("type")
        if el.get("isDeleted") or not _visible(el) or kind not in ("rectangle", "ellipse", "diamond", "line"):
            continue
        if kind == "line":
            poly = _path(el)
            if len(poly) < 4 or math.dist(poly[0], poly[-1]) > 1:
                continue  # an open line: a divider or a cylinder side
            xs, ys = [p[0] for p in poly], [p[1] for p in poly]
            shapes[el_id] = Shape(el_id, "polygon", (min(xs), min(ys), max(xs), max(ys)), poly + poly[:1], _group(el))
            continue
        x, y, w, h = _num(el, "x"), _num(el, "y"), _num(el, "width"), _num(el, "height")
        box = (min(x, x + w), min(y, y + h), max(x, x + w), max(y, y + h))
        if kind == "diamond":
            cx, cy = (box[0] + box[2]) / 2, (box[1] + box[3]) / 2
            poly = [(cx, box[1]), (box[2], cy), (cx, box[3]), (box[0], cy), (cx, box[1])]
            shapes[el_id] = Shape(el_id, "polygon", box, poly, _group(el))
            continue
        labeled = any(b.get("type") == "text" for b in _bound_items(el))
        badge = kind == "ellipse" and labeled and abs(w) <= BADGE_MAX and abs(h) <= BADGE_MAX
        shapes[el_id] = Shape(el_id, kind, box, group=_group(el), badge=badge)
    return shapes


class TextBox:
    """Where free text or an arrow label renders, by Excalidraw's font widths."""

    def __init__(self, el_id: str, box: Box, arrow: str | None):
        self.id, self.box, self.arrow = el_id, box, arrow


def _arrow_label_box(el: dict, arrow: dict) -> Box | None:
    path = _path(arrow)
    if len(path) < 2:
        return None
    (cx, cy), w, h = _label_point(path), text_width(el), needed_height(el)
    return cx - w / 2, cy - h / 2, cx + w / 2, cy + h / 2


def text_boxes(by_id: dict[str, dict]) -> list[TextBox]:
    boxes = []
    for el_id, el in by_id.items():
        if el.get("type") != "text" or el.get("isDeleted") or not str(el.get("text", "")).strip():
            continue
        container = by_id.get(_ref(el, "containerId") or "")
        if container is None:
            boxes.append(TextBox(el_id, _free_text_box(el), None))
        elif container.get("type") == "arrow" and _arrow_label_box(el, container):
            boxes.append(TextBox(el_id, _arrow_label_box(el, container), container["id"]))
    return boxes


def _group_box(shape: Shape, by_id: dict[str, dict]) -> Shape | None:
    """The box around every drawn part of shape's group, such as a whole cylinder."""
    if not shape.group:
        return None
    boxes = []
    for el in by_id.values():
        if _group(el) != shape.group or el.get("isDeleted") or el.get("type") in ("text", "arrow"):
            continue
        path = _path(el) if el.get("type") in POINT_TYPES else []
        if path:
            boxes.append((min(p[0] for p in path), min(p[1] for p in path), max(p[0] for p in path), max(p[1] for p in path)))
        else:
            x, y = _num(el, "x"), _num(el, "y")
            boxes.append((x, y, x + _num(el, "width"), y + _num(el, "height")))
    box = (min(b[0] for b in boxes), min(b[1] for b in boxes), max(b[2] for b in boxes), max(b[3] for b in boxes))
    return Shape(shape.group, "rectangle", box)


def _end_target(arrow: dict, key: str, by_id: dict[str, dict], shapes: dict[str, Shape]) -> Shape | None:
    binding = arrow.get(key)
    target = _ref(binding, "elementId") if isinstance(binding, dict) else None
    if target in shapes:
        return shapes[target]
    el = by_id.get(target or "")
    if el is None or el.get("type") != "text":
        return None
    x, y = _num(el, "x"), _num(el, "y")
    return Shape(target, "rectangle", (x, y, x + _num(el, "width"), y + _num(el, "height")))


def _check_overlaps(parts: list[Shape], parent: dict[str, str | None]) -> list[str]:
    pairs = set()
    for i, a in enumerate(parts):
        for b in parts[i + 1:]:
            if a.group and a.group == b.group or _holds(a.box, b.box) or _holds(b.box, a.box):
                continue
            if _overlap(a.box, b.box) > EDGE_MARGIN:
                pairs.add((a.id, b.id))
    # When a box overlaps, its children overlap too; report only the box.
    reported = {frozenset(pair) for pair in pairs}
    return [f"'{a}' and '{b}' overlap, and neither holds the other; move them apart, or put one fully inside the other"
            for a, b in sorted(pairs)
            if frozenset((parent[a], b)) not in reported and frozenset((a, parent[b])) not in reported]


def _edge_hit(shape: Shape, path: list[Point]) -> tuple[float, Point]:
    """How far the path's last point is from where the path meets shape's edge, and that point."""
    end = path[-1]
    near = shape.nearest_edge(end)
    if shape.inside(end):
        # Walk back along the arrow to where it entered the shape.
        outside = next((q for q in reversed(_samples(path)) if not shape.inside(q)), None)
        edge = shape.nearest_edge(outside) if outside else near
        return math.dist(edge, end), edge
    gap = math.dist(near, end)
    if gap <= END_SLACK:
        return gap, near
    # Follow the last segment forward, in case it points at the shape.
    prev = path[-2]
    length = math.dist(prev, end)
    if length:
        ux, uy = (end[0] - prev[0]) / length, (end[1] - prev[1]) / length
        for step in range(1, 201):
            q = (end[0] + ux * step, end[1] + uy * step)
            if shape.inside(q):
                edge = shape.nearest_edge(q)
                return math.dist(edge, end), edge
    return gap, near


def _check_arrow_ends(arrows: list[dict], by_id: dict[str, dict], shapes: dict[str, Shape]) -> list[str]:
    warnings = []
    for arrow in arrows:
        path = _path(arrow)
        for key, toward, verb in (("startBinding", path[::-1], "starts"), ("endBinding", path, "ends")):
            shape = _end_target(arrow, key, by_id, shapes)
            if shape is None:
                continue
            gap, edge = _edge_hit(shape, toward)
            whole = _group_box(shape, by_id)
            if gap <= END_SLACK or whole and _edge_hit(whole, toward)[0] <= END_SLACK:
                continue
            side = "inside" if shape.inside(toward[-1]) else "outside"
            warnings.append(f"'{arrow['id']}' {verb} {gap:.0f} px {side} the edge of '{shape.id}'; "
                            f"move that end to about ({edge[0]:.0f}, {edge[1]:.0f}), on the edge")
    return warnings


def _marks(shape: Shape, path: list[Point]) -> bool:
    """A dot centered on the path is drawn on the arrow, not crossed by it."""
    x0, y0, x1, y1 = shape.box
    small = x1 - x0 <= MARK_MAX and y1 - y0 <= MARK_MAX
    return small and _distance_along(path, ((x0 + x1) / 2, (y0 + y1) / 2))[0] <= BADGE_SLACK


def _check_arrow_paths(arrows: list[dict], by_id: dict[str, dict], shapes: dict[str, Shape],
                       holders: set[str], texts: list[TextBox]) -> list[str]:
    warnings = []
    for arrow in arrows:
        path = _path(arrow)
        points = _samples(path)
        reach = (min(p[0] for p in path), min(p[1] for p in path), max(p[0] for p in path), max(p[1] for p in path))
        # An arrow may cross the shapes at its ends: the bound ones and their groups or, at an
        # unbound end, any shape it starts or stops in or on.
        ends: set[str] = set()
        for key, p in (("startBinding", path[0]), ("endBinding", path[-1])):
            target = _end_target(arrow, key, by_id, shapes)
            if target:
                ends.add(target.id)
            else:
                ends |= {s.id for s in shapes.values() if s.inside(p) or math.dist(s.nearest_edge(p), p) <= END_SLACK}
        end_groups = {shapes[s].group for s in ends if s in shapes and shapes[s].group}
        for shape in shapes.values():
            # A boundary may hold the arrow's start or end; an arrow through any other boundary crosses it.
            holds_end = shape.id in holders and (shape.inside(path[0]) or shape.inside(path[-1]))
            if (shape.id in ends or holds_end or shape.badge or shape.group in end_groups
                    or (shape.group and shape.group == _group(arrow)) or _overlap(shape.box, reach) < 0
                    or _marks(shape, path)):
                continue
            hit = next((p for p in points if shape.inside(p, EDGE_MARGIN)), None)
            if hit:
                warnings.append(f"'{arrow['id']}' passes through '{shape.id}' near ({hit[0]:.0f}, {hit[1]:.0f}); route it around '{shape.id}'")
        for text in texts:
            if text.arrow == arrow["id"] or _overlap(text.box, reach) < 0:
                continue
            hit = next((p for p in points if _in_box(text.box, p, 1)), None)
            if hit:
                warnings.append(f"'{arrow['id']}' runs through text '{text.id}' near ({hit[0]:.0f}, {hit[1]:.0f}); move the text off the arrow's path")
    return warnings


def _crosses(box: Box, shape: Shape) -> bool:
    """True when the shape's edge runs through the text box, so the text is neither inside nor clear."""
    return _overlap(box, shape.box) > 1 and any(_in_box(box, p, 1) for p in shape.outline())


def _label_span(path: list[Point], curved: bool) -> tuple[Point, Point, float, float] | None:
    """The stretch of path a bound label sits on: its two ends, its length, and how far along the
    path it starts. None when the label sits on a sharp corner."""
    lengths = [math.dist(a, b) for a, b in zip(path, path[1:])]
    if len(path) % 2 == 0:
        i = len(path) // 2 - 1
        return path[i], path[i + 1], lengths[i], sum(lengths[:i])
    if not curved:
        return None
    m = len(path) // 2
    return path[m - 1], path[m + 1], lengths[m - 1] + lengths[m], sum(lengths[:m - 1])


def _check_arrow_labels(arrows: list[dict], by_id: dict[str, dict], shapes: dict[str, Shape]) -> list[str]:
    labels: dict[str, list[dict]] = {}
    for el in by_id.values():
        if el.get("type") == "text" and not el.get("isDeleted") and _ref(el, "containerId"):
            labels.setdefault(el["containerId"], []).append(el)
    badges = [s for s in shapes.values() if s.badge]
    warnings = []
    for arrow in arrows:
        path = _path(arrow)
        curved = isinstance(arrow.get("roundness"), dict) and not arrow.get("elbowed")
        span = _label_span(path, curved)
        for label in labels.get(arrow["id"], []):
            if span is None:
                warnings.append(f"'{label['id']}' sits on a corner of arrow '{arrow['id']}', which splits the line there; "
                                f"give the arrow 2 or 4 points, or put free text beside it")
                continue
            a, b, length, start = span
            angle = math.atan2(b[1] - a[1], b[0] - a[0])
            along = text_width(label) * abs(math.cos(angle)) + needed_height(label) * abs(math.sin(angle))
            badge = next((s.id for s in badges if _on_span(s, path, start, length)), None)
            need = along + (BADGE_ROOM if badge else ARROW_SLACK)
            if length >= need:
                continue
            where = f"arrow '{arrow['id']}' is" if len(path) == 2 else f"the part of arrow '{arrow['id']}' that holds it is"
            why = f", since badge '{badge}' sits there too" if badge else ""
            warnings.append(f"'{label['id']}' needs about {along:.0f} px, but {where} {length:.0f} px long; make it at least {need:.0f} px{why}")
    return warnings


def _on_span(badge: Shape, path: list[Point], start: float, length: float) -> bool:
    center = ((badge.box[0] + badge.box[2]) / 2, (badge.box[1] + badge.box[3]) / 2)
    distance, along = _distance_along(path, center)
    return distance <= BADGE_SLACK and start <= along <= start + length


def _check_texts(texts: list[TextBox], shapes: dict[str, Shape]) -> list[str]:
    warnings = []
    for i, text in enumerate(texts):
        hit_groups = set()
        for shape in shapes.values():
            if shape.group in hit_groups or not _crosses(text.box, shape):
                continue
            if shape.group:
                hit_groups.add(shape.group)
            if text.arrow:
                warnings.append(f"label '{text.id}' of arrow '{text.arrow}' covers part of '{shape.id}'; lengthen or reroute the arrow, or move '{shape.id}'")
            elif shape.badge:
                warnings.append(f"text '{text.id}' covers part of badge '{shape.id}'; move the badge or the text")
            else:
                warnings.append(f"text '{text.id}' covers part of '{shape.id}'; move it fully inside '{shape.id}' or fully clear of it")
        for other in texts[i + 1:]:
            if _overlap(text.box, other.box) > 1:
                warnings.append(f"texts '{text.id}' and '{other.id}' overlap; move one")
    return warnings


def _check_badges(shapes: dict[str, Shape], arrows: list[dict]) -> list[str]:
    warnings = []
    paths = [(arrow["id"], _path(arrow)) for arrow in arrows]
    for badge in (s for s in shapes.values() if s.badge):
        center = ((badge.box[0] + badge.box[2]) / 2, (badge.box[1] + badge.box[3]) / 2)
        spots = []
        for arrow_id, path in paths:
            distance, along = _distance_along(path, center)
            if distance <= BADGE_SLACK:
                spots.append((abs(along - BADGE_OFFSET), along, arrow_id))
        if spots and min(spots)[0] > BADGE_SLACK:
            _, along, arrow_id = min(spots)
            warnings.append(f"badge '{badge.id}' sits {along:.0f} px along '{arrow_id}' from its start; center it {BADGE_OFFSET} px from the start, clear of the label")
    return warnings


def _check_label_room(parts: list[Shape], parent: dict[str, str | None], texts: list[TextBox]) -> list[str]:
    by_id = {s.id: s for s in parts}
    labeled = set()
    for text in (t for t in texts if not t.arrow):
        holders = [s for s in parts if _within(text.box, s.box, 0)]
        if holders:
            holder = min(holders, key=lambda s: _area(s.box))
            if text.box[1] - holder.box[1] <= LABEL_BAND:
                labeled.add(holder.id)
    warnings = []
    hit_groups = set()
    for shape in sorted(parts, key=lambda s: s.box[1]):
        if shape.group in hit_groups:
            continue
        outer = by_id.get(parent[shape.id] or "")
        if outer and outer.id in labeled and shape.box[1] - outer.box[1] < LABEL_ROOM:
            if shape.group:
                hit_groups.add(shape.group)
            warnings.append(f"'{shape.id}' starts {shape.box[1] - outer.box[1]:.0f} px below the top of '{outer.id}'; "
                            f"start it at least {LABEL_ROOM} px down, clear of the label")
    return warnings


def check_layout(by_id: dict[str, dict]) -> list[str]:
    """Warnings for overlaps, arrow ends, arrow paths, text placement, badges, and label room."""
    shapes = shapes_of(by_id)
    parts = [s for s in shapes.values() if not s.badge]
    parent: dict[str, str | None] = {}
    for shape in parts:
        holders = [o for o in parts if o is not shape and _holds(o.box, shape.box)]
        parent[shape.id] = min(holders, key=lambda o: _area(o.box)).id if holders else None
    holders = {p for p in parent.values() if p}
    arrows = [el for el in by_id.values()
              if el.get("type") == "arrow" and not el.get("isDeleted") and len(_path(el)) >= 2]
    texts = text_boxes(by_id)
    return (_check_overlaps(parts, parent) + _check_arrow_labels(arrows, by_id, shapes)
            + _check_arrow_ends(arrows, by_id, shapes)
            + _check_arrow_paths(arrows, by_id, shapes, holders, texts) + _check_texts(texts, shapes)
            + _check_badges(shapes, arrows) + _check_label_room(parts, parent, texts))


def bounds(elements: list) -> tuple[float, float, float, float] | None:
    by_id = {el["id"]: el for el in elements if isinstance(el, dict) and isinstance(el.get("id"), str)}
    xs: list[float] = []
    ys: list[float] = []
    for el in elements:
        if not isinstance(el, dict) or el.get("isDeleted"):
            continue
        container = by_id.get(_ref(el, "containerId") or "") if el.get("type") == "text" else None
        if container and container.get("type") == "arrow":
            # The stored x/y of an arrow label do not matter; Excalidraw draws it on the arrow.
            box = _arrow_label_box(el, container)
            if box:
                xs += [box[0], box[2]]
                ys += [box[1], box[3]]
            continue
        x, y = _num(el, "x"), _num(el, "y")
        points = _path(el) if el.get("type") in POINT_TYPES else []
        if points:
            xs += [p[0] for p in points]
            ys += [p[1] for p in points]
        else:
            xs += [x, x + _num(el, "width")]
            ys += [y, y + _num(el, "height")]
    if not xs:
        return None
    return min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys)


def check(data) -> tuple[list[str], list[str]]:
    """Return (errors, warnings) for parsed Excalidraw JSON."""
    if not isinstance(data, dict):
        return ["the file is not a JSON object"], []
    errors: list[str] = []
    if data.get("type") != "excalidraw":
        errors.append(f"expected type 'excalidraw', got {data.get('type')!r}")
    elements = data.get("elements")
    if not isinstance(elements, list) or not elements:
        errors.append("'elements' is missing, empty, or not an array")
        return errors, []
    by_id, index_errors = index(elements)
    element_errors, warnings = check_elements(by_id)
    return errors + index_errors + element_errors, warnings + check_layout(by_id)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check an .excalidraw file's ids, bindings, text fit, and layout.")
    parser.add_argument("file", type=Path, help="path to the .excalidraw file")
    args = parser.parse_args(argv)
    try:
        data = json.loads(args.file.read_text(encoding="utf-8"))
    except OSError as e:
        print(f"ERROR: cannot read {args.file}: {e.strerror or e}")
        return 2
    except ValueError as e:
        print(f"ERROR: {args.file} is not valid JSON: {e}")
        return 1

    errors, warnings = check(data)
    for message in errors:
        print(f"ERROR: {message}")
    for message in warnings:
        print(f"WARNING: {message}")
    elements = data.get("elements") if isinstance(data, dict) else None
    box = bounds(elements) if isinstance(elements, list) else None
    if box:
        print("Diagram bounds: x={:.0f} y={:.0f} w={:.0f} h={:.0f}".format(*box))
    print(f"{len(errors)} errors, {len(warnings)} warnings")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

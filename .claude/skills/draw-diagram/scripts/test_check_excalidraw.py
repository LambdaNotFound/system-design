#!/usr/bin/env python3
"""Tests for check_excalidraw.py. Run: python3 -B test_check_excalidraw.py"""

from __future__ import annotations

import contextlib
import copy
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_excalidraw  # noqa: E402

BASE = {
    "type": "excalidraw",
    "version": 2,
    "elements": [
        {"type": "rectangle", "id": "api", "x": 0, "y": 0, "width": 180, "height": 70,
         "boundElements": [{"id": "api_label", "type": "text"}, {"id": "a1", "type": "arrow"}]},
        {"type": "text", "id": "api_label", "x": 40, "y": 25, "width": 100, "height": 20,
         "text": "Order API", "fontSize": 16, "textAlign": "center", "containerId": "api"},
        {"type": "rectangle", "id": "db", "x": 300, "y": 0, "width": 180, "height": 70,
         "boundElements": [{"id": "a1", "type": "arrow"}]},
        {"type": "arrow", "id": "a1", "x": 180, "y": 35, "width": 120, "height": 0,
         "points": [[0, 0], [120, 0]],
         "startBinding": {"elementId": "api"}, "endBinding": {"elementId": "db"},
         "boundElements": [{"id": "a1_label", "type": "text"}]},
        {"type": "text", "id": "a1_label", "x": 220, "y": 26, "width": 30, "height": 18,
         "text": "SQL", "fontSize": 14, "containerId": "a1"},
    ],
    "appState": {"viewBackgroundColor": "#ffffff"},
    "files": {},
}


def element(data: dict, el_id: str) -> dict:
    return next(el for el in data["elements"] if el["id"] == el_id)


class CheckTest(unittest.TestCase):
    def run_check(self, data=None, raw: str | None = None, missing: bool = False) -> tuple[int, str]:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "d.excalidraw"
            if not missing:
                path.write_text(raw if raw is not None else json.dumps(data), encoding="utf-8")
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                code = check_excalidraw.main([str(path)])
            return code, out.getvalue()

    def diagram(self) -> dict:
        return copy.deepcopy(BASE)

    def test_clean_diagram_passes_and_prints_bounds(self):
        code, out = self.run_check(self.diagram())
        self.assertEqual(code, 0, out)
        self.assertIn("Diagram bounds: x=0 y=0 w=480 h=70", out)
        self.assertIn("0 errors, 0 warnings", out)

    def test_binding_listed_on_one_end_is_an_error(self):
        data = self.diagram()
        element(data, "db")["boundElements"] = []
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("'a1'.endBinding -> 'db', but 'db'.boundElements does not list 'a1'", out)

    def test_duplicate_and_missing_ids_are_errors(self):
        data = self.diagram()
        data["elements"].append(copy.deepcopy(element(data, "db")))
        element(data, "api")["boundElements"].append({"id": "ghost", "type": "arrow"})
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("duplicate id 'db'", out)
        self.assertIn("'api'.boundElements lists missing id 'ghost'", out)

    def test_text_too_tall_or_off_center_is_an_error(self):
        data = self.diagram()
        element(data, "api_label").update(y=0, height=70)
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("renders at the top of its shape", out)
        element(data, "api_label").update(x=400, y=25, height=20)
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("its center is outside that shape", out)

    def test_wide_text_warns_without_failing(self):
        data = self.diagram()
        element(data, "api_label")["text"] = "Notification Service Gateway"
        code, out = self.run_check(data)
        self.assertEqual(code, 0, out)
        self.assertIn("WARNING: 'api_label' needs about 205 px, but rectangle 'api' leaves 140 px; widen 'api' by 65 px", out)
        self.assertIn("0 errors, 1 warnings", out)

    def test_a_diamond_leaves_less_room_than_a_rectangle(self):
        data = self.diagram()
        element(data, "api_label")["text"] = "Payment approved"
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("0 errors, 0 warnings", out)
        element(data, "api")["type"] = "diamond"
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("diamond 'api' leaves 129 px", out)

    def test_a_label_longer_than_its_arrow_warns(self):
        data = self.diagram()
        element(data, "a1_label")["text"] = "publish order.created"
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("'a1_label' needs about 132 px, but arrow 'a1' is 120 px long; make it at least 192 px", out)

    def test_text_in_a_line_is_an_error(self):
        data = self.diagram()
        element(data, "a1")["type"] = "line"
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("a line; only rectangle, ellipse, diamond, and arrow can hold text", out)

    def test_wrong_type_and_no_elements_are_errors(self):
        code, out = self.run_check({"type": "drawing", "elements": []})
        self.assertEqual(code, 1)
        self.assertIn("expected type 'excalidraw', got 'drawing'", out)
        self.assertIn("'elements' is missing, empty, or not an array", out)

    def test_a_non_numeric_field_is_an_error_not_a_crash(self):
        data = self.diagram()
        element(data, "db")["x"] = "300"
        code, out = self.run_check(data)
        self.assertEqual(code, 1)
        self.assertIn("'db' has non-numeric x", out)

    def test_shapes_that_overlap_without_nesting_warn(self):
        data = self.diagram()
        data["elements"].append({"type": "rectangle", "id": "cache", "x": 100, "y": 50, "width": 60, "height": 60})
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("'api' and 'cache' overlap, and neither holds the other", out)

    def test_nested_shapes_and_parts_of_one_group_do_not_warn(self):
        data = self.diagram()
        data["elements"].insert(0, {"type": "rectangle", "id": "zone", "x": -40, "y": -60, "width": 560, "height": 180})
        data["elements"].insert(1, {"type": "rectangle", "id": "api_back", "x": 8, "y": -8, "width": 180,
                                    "height": 70, "groupIds": ["api_stack"]})
        element(data, "api")["groupIds"] = ["api_stack"]
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)

    def test_an_arrow_end_off_its_shape_warns(self):
        data = self.diagram()
        element(data, "db")["x"] = 340
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("'a1' ends 40 px outside the edge of 'db'; move that end to about (340, 35), on the edge", out)
        element(data, "a1")["points"] = [[0, 0], [200, 0]]
        code, out = self.run_check(data)
        self.assertIn("'a1' ends 40 px inside the edge of 'db'; move that end to about (340, 35), on the edge", out)

    def test_an_arrow_may_end_on_the_edge_of_its_shapes_group(self):
        data = self.diagram()
        element(data, "db").update(groupIds=["db_group"], boundElements=[{"id": "a1", "type": "arrow"}, {"id": "a2", "type": "arrow"}])
        data["elements"] += [
            {"type": "ellipse", "id": "db_top", "x": 300, "y": -14, "width": 180, "height": 28, "groupIds": ["db_group"]},
            {"type": "arrow", "id": "a2", "x": 390, "y": -100, "width": 0, "height": 82, "points": [[0, 0], [0, 82]],
             "endBinding": {"elementId": "db"}},
        ]
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)
        element(data, "db")["groupIds"] = []
        code, out = self.run_check(data)
        self.assertIn("'a2' ends 18 px outside the edge of 'db'", out)

    def test_an_arrow_through_a_shape_or_free_text_warns(self):
        data = self.diagram()
        data["elements"] += [
            {"type": "rectangle", "id": "cache", "x": 190, "y": 10, "width": 30, "height": 50},
            {"type": "text", "id": "note", "x": 260, "y": 28, "width": 30, "height": 18, "text": "note", "fontSize": 14},
        ]
        code, out = self.run_check(data)
        self.assertEqual(code, 0)
        self.assertIn("'a1' passes through 'cache' near (194, 35); route it around 'cache'", out)
        self.assertIn("'a1' runs through text 'note' near (262, 35)", out)
        self.assertIn("0 errors, 2 warnings", out)
        element(data, "cache").update(x=195, y=30, width=10, height=10)
        code, out = self.run_check(data)
        self.assertNotIn("'cache'", out)

    def test_text_across_a_shape_edge_or_over_other_text_warns(self):
        data = self.diagram()
        data["elements"] += [
            {"type": "text", "id": "caption", "x": 150, "y": 60, "width": 60, "height": 17, "text": "orders", "fontSize": 13},
            {"type": "text", "id": "note1", "x": 0, "y": 100, "width": 60, "height": 17, "text": "first", "fontSize": 13},
            {"type": "text", "id": "note2", "x": 10, "y": 108, "width": 60, "height": 17, "text": "second", "fontSize": 13},
        ]
        code, out = self.run_check(data)
        self.assertIn("text 'caption' covers part of 'api'; move it fully inside 'api' or fully clear of it", out)
        self.assertIn("texts 'note1' and 'note2' overlap", out)

    def test_a_badge_off_its_spot_warns(self):
        data = self.diagram()
        element(data, "db")["x"] = 340
        element(data, "a1").update(width=160, points=[[0, 0], [160, 0]])
        data["elements"] += [
            {"type": "ellipse", "id": "step1", "x": 190, "y": 21, "width": 28, "height": 28,
             "boundElements": [{"id": "step1_label", "type": "text"}]},
            {"type": "text", "id": "step1_label", "x": 200, "y": 26, "width": 8, "height": 18,
             "text": "1", "fontSize": 14, "containerId": "step1"},
        ]
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)
        element(data, "step1")["x"] = 266
        element(data, "step1_label")["x"] = 276
        code, out = self.run_check(data)
        self.assertIn("badge 'step1' sits 100 px along 'a1' from its start; center it 24 px from the start", out)

    def test_a_child_too_close_to_a_labeled_boundary_top_warns(self):
        data = self.diagram()
        data["elements"][:0] = [
            {"type": "rectangle", "id": "zone", "x": -40, "y": -40, "width": 560, "height": 160},
            {"type": "text", "id": "zone_label", "x": -24, "y": -28, "width": 40, "height": 20, "text": "Zone", "fontSize": 16},
        ]
        code, out = self.run_check(data)
        self.assertIn("'api' starts 40 px below the top of 'zone'; start it at least 44 px down", out)
        self.assertIn("'db' starts 40 px below the top of 'zone'", out)
        element(data, "zone").update(y=-44, height=164)
        element(data, "zone_label")["y"] = -32
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)

    def test_bounds_place_an_arrow_label_on_its_arrow(self):
        data = self.diagram()
        element(data, "a1_label").update(x=-3000, y=-3000)
        code, out = self.run_check(data)
        self.assertIn("Diagram bounds: x=0 y=0 w=480 h=70", out)

    def test_a_label_needs_room_along_its_own_segment(self):
        data = self.diagram()
        element(data, "a1_label")["text"] = "INSERT order"
        element(data, "db").update(x=0, y=170)
        element(data, "a1").update(x=90, y=74, points=[[0, 0], [0, 92]])
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)
        element(data, "db").update(x=300, y=10)
        element(data, "a1").update(x=180, y=35, points=[[0, 0], [60, 0], [60, 10], [120, 10]])
        code, out = self.run_check(data)
        self.assertIn("the part of arrow 'a1' that holds it is 10 px long; make it at least 78 px", out)

    def test_a_label_on_a_sharp_corner_warns_but_not_on_a_curve(self):
        data = self.diagram()
        element(data, "db").update(x=300, y=200)
        element(data, "a1").update(x=90, y=74, points=[[0, 0], [0, 161], [206, 161]], roundness=None)
        code, out = self.run_check(data)
        self.assertIn("'a1_label' sits on a corner of arrow 'a1'", out)
        element(data, "a1")["roundness"] = {"type": 2}
        code, out = self.run_check(data)
        self.assertNotIn("sits on a corner", out)

    def test_a_badge_on_the_label_segment_needs_100_px_more(self):
        data = self.diagram()
        data["elements"] += [
            {"type": "ellipse", "id": "step1", "x": 190, "y": 21, "width": 28, "height": 28,
             "boundElements": [{"id": "step1_label", "type": "text"}]},
            {"type": "text", "id": "step1_label", "x": 200, "y": 26, "width": 8, "height": 18,
             "text": "1", "fontSize": 14, "containerId": "step1"},
        ]
        code, out = self.run_check(data)
        self.assertIn("arrow 'a1' is 120 px long; make it at least 128 px, since badge 'step1' sits there too", out)

    def test_widen_advice_fits_a_diamond_in_one_step(self):
        data = self.diagram()
        element(data, "api")["type"] = "diamond"
        element(data, "api_label").update(text="Payment approved", x=23, width=134)
        code, out = self.run_check(data)
        self.assertIn("diamond 'api' leaves 129 px; widen 'api' by 9 px", out)
        element(data, "api")["width"] = 189
        element(data, "api_label")["x"] = 27.5
        element(data, "a1").update(x=189, width=111, points=[[0, 0], [111, 0]])
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)

    def test_text_near_a_diamond_follows_its_outline(self):
        data = self.diagram()
        data["elements"] += [
            {"type": "diamond", "id": "d", "x": 0, "y": 200, "width": 160, "height": 100},
            {"type": "text", "id": "yes", "x": 140, "y": 194, "width": 22, "height": 18, "text": "yes", "fontSize": 14},
            {"type": "text", "id": "note", "x": 104, "y": 212, "width": 30, "height": 18, "text": "note", "fontSize": 14},
        ]
        code, out = self.run_check(data)
        self.assertNotIn("'yes'", out)
        self.assertIn("text 'note' covers part of 'd'", out)

    def test_a_bound_label_off_center_warns(self):
        data = self.diagram()
        element(data, "api_label")["x"] = 0
        code, out = self.run_check(data)
        self.assertIn("'api_label' sits 40 px off the center of 'api'; set x = 40, y = 25", out)

    def test_a_label_and_a_second_line_inside_the_shape_center_as_one_block(self):
        data = self.diagram()
        element(data, "api_label")["y"] = 16
        data["elements"].append({"type": "text", "id": "api_tech", "x": 40, "y": 38, "width": 100, "height": 16,
                                 "text": "Go, :8080", "fontSize": 13, "textAlign": "center"})
        code, out = self.run_check(data)
        self.assertIn("0 errors, 0 warnings", out)

    def test_an_arrow_through_a_legend_it_does_not_start_or_end_in_warns(self):
        data = self.diagram()
        data["elements"][:0] = [
            {"type": "rectangle", "id": "legend", "x": 200, "y": -40, "width": 60, "height": 150},
            {"type": "rectangle", "id": "swatch", "x": 210, "y": -30, "width": 28, "height": 18},
        ]
        code, out = self.run_check(data)
        self.assertIn("'a1' passes through 'legend'", out)

    def test_wrong_field_types_are_errors_not_crashes(self):
        cases = [("api", "groupIds", 5), ("api", "boundElements", 5), ("a1", "points", 5),
                 ("a1", "endBinding", {"elementId": ["db"]}), ("api_label", "fontFamily", [9]),
                 ("api_label", "containerId", ["api"]), ("db", "frameId", ["f"])]
        for el_id, key, value in cases:
            data = self.diagram()
            element(data, el_id)[key] = value
            code, out = self.run_check(data)
            self.assertEqual(code, 1, (key, out))
            self.assertIn(f"'{el_id}' has a bad {key}", out)
        code, out = self.run_check({"type": "excalidraw", "elements": 5})
        self.assertEqual(code, 1)
        self.assertIn("'elements' is missing, empty, or not an array", out)

    def test_bad_json_exits_1_and_a_missing_file_exits_2(self):
        code, out = self.run_check(raw="{not json")
        self.assertEqual(code, 1)
        self.assertIn("is not valid JSON", out)
        code, out = self.run_check(missing=True)
        self.assertEqual(code, 2)
        self.assertIn("cannot read", out)


if __name__ == "__main__":
    unittest.main()

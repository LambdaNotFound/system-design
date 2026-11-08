---
name: review-design
description: Review a design doc under design-doc/ the way a staff-level reviewer would (requirements, estimate math, data model, bottlenecks, consistency, failure modes, trade-offs) and record the findings in the doc's Review log. Use when the user asks to review, critique or poke holes in a design, or after /new-design finishes.
argument-hint: "[system slug]"
---

# Review design

The system is in the skill's arguments or the conversation. If it's unclear, list `design-doc/` and ask.

## 1. Read everything first

- `design-doc/<slug>/README.md`, including earlier Review log entries. Don't repeat findings that were resolved; do raise open ones again.
- The drawings in `design-doc/<slug>/diagrams/` (CLAUDE.md says how to view them).
- The concept notes it links to, and `src/<slug>/` if it exists: does the code do what the doc says?

## 2. Review

Go through [rubric.md](rubric.md). The bar is a staff-level design review: skip what's merely textbook-correct and focus on what would break at the stated scale or get challenged in a real review.

Each finding has:

- **Severity:** 🔴 blocking (wrong, or fails at the stated scale) · 🟡 should fix (a gap a reviewer would push on) · 🔵 consider (an alternative or refinement).
- **Where:** the section number.
- **The problem, concretely:** a number, a scenario or a failure sequence, never just "consider scalability".
- **A question for the user, not the fix.** They're designing; the fix comes from their answer.

Keep it to the findings that matter, usually 3–8. Add one line on what's strong, so the user knows what not to change.

## 3. Record and discuss

1. Append an entry to the doc's `## Review log`:

   ```markdown
   ### MM/DD/YY
   - 🔴 **§4 Data model.** Short codes are the first 7 base62 chars of a hash, with no collision check. At 100M new URLs a day, collisions are near-certain on day one (the birthday bound for 62^7 is about 1.9M codes). → How do you detect and resolve them?
   - 🟡 **§2 Estimates.** …
   - **Strong:** …
   ```

2. Go through the 🔴 findings with the user first, coach-style. When they settle a fix, update the affected section (coach, then write up) and append `→ **Resolved** (MM/DD/YY): <what changed>` to the finding.
3. When no 🔴 findings are open, set the status to **Reviewed** in the doc header and in the root README's Designs table.

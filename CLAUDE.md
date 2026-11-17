# CLAUDE.md

System-design learning notes: design docs, concept notes, and small Rust implementations. Claude is the user's **coach** and a **co-author** here; "How we work" says which role applies where.

## Layout

```
.claude/skills/     scope-problem · new-design · review-design · implement-design ·
                    draw-diagram · write-concept · drill
.githooks/          commit-msg checks the commit format; prepare-commit-msg adds the date;
                    pre-commit lints the skills; post-commit, post-merge and post-rewrite
                    install them into ~/.claude/skills when on main
evals/              shared fixtures and routing cases for the skill evals; each skill's own
                    cases are in .claude/skills/<skill>/evals/
scripts/            check-skills (lint) · eval-skills (runs the eval cases) ·
                    sync-skills (installs the skills for every project)
concepts/           <slug>.md notes on building blocks and theory, linked from the designs;
                    <slug>/ holds the implementation loop's files when a concept has a crate
design-doc/<slug>/  requirement.md is the signed-off requirements; scoping.md is its working record;
                    design.md is the RFC with its loop state; review.md is the review ledger;
                    rfc.md is the exported RFC; diagrams/ holds its Excalidraw drawings;
                    impl.md is the crate's plan and log; impl-review.md is the code review ledger
drills/<slug>.md    the user's recall record for a note or a design; git-ignored, never committed
src/                Cargo workspace; src/<slug>/ is one crate per system or concept
README.md           index of all designs (status · started · code)
```

- Each system or concept has one kebab-case slug (`url-shortener`), used unchanged in folder names, the README index and commit messages. No number prefixes.
- Design docs link to concept notes instead of re-explaining a building block.

## How we work

- **Scoping: interview, then sign off.** `/scope-problem` comes before a design. Claude interviews the user in themed rounds, five to eight for a system: numbered questions that each carry a proposed default to correct, then a dialog for the round's trade-offs. It checks the draft for clashes and gaps after every round, reads this repo and the sources the user names, and writes `requirement.md` for the user to sign. The RFC carries its FR and NFR rows verbatim and cites them by ID.
- **Design docs: an agent loop writes the RFC.** `/new-design` takes the signed requirement doc and runs a design-review loop of fresh subagents: an investigator writes the design, a bar-raiser challenges it in rounds until it approves or escalates, and an auditor gives the approved text one editorial pass. Claude leads the loop and writes no design itself. The user decides what the loop escalates. Don't hand-write or hand-edit `design.md`, `review.md` or `rfc.md`.
- **Reviews: on demand, by a fresh bar-raiser.** `/review-design` critiques a design from this repo or any design doc by path, with an optional focus area. Findings go to the design's `review.md` with IDs. Go through them with the user as a coach; a design changes only when the user picks the objections to fix.
- **Concept notes: Claude writes them.** `/write-concept` writes `concepts/<slug>.md` from primary sources it reads in the session, with check-your-understanding questions and a sources list that says what was read and what is cited from memory. The user annotates under My notes, which Claude never edits. It links a note to the designs that use it, and never edits a design to link back. Index: `concepts/README.md`.
- **Recall: Claude asks, the user answers.** `/drill` questions the user on a concept note or a design, one question at a time: a failure to predict, a number to change, a decision to defend. It grades each answer against the doc, never against memory, and asks one follow-up where the answer was thin. The record in `drills/` brings back what was missed. It is private: don't commit it or quote it into a tracked file.
- **Code: an agent loop writes the crate, tests first.** `/implement-design` takes the RFC and runs an implementation loop of fresh subagents: a coding expert plans the crate and the user approves the plan; it writes the types and failing tests, which a bar-raiser reviews against the design; then it writes the logic, which the bar-raiser reviews against the checks Claude ran. Claude leads the loop and writes no code itself. The user decides what the loop escalates and reads the finished crate by hand. Don't hand-edit `impl.md` or `impl-review.md`.
- **Diagrams:** Claude writes sequence and flow diagrams in Mermaid, inline in the doc. Architecture diagrams are `diagrams/*.excalidraw.svg`, drawn by the user after the design is approved: the RFC carries a brief of what to draw, and Claude reviews the drawing before it is embedded. To look at a drawing on macOS, render it with `qlmanage -t -s 1600 -o <scratchpad> <file>` and read the PNG; elsewhere, read the SVG's `<text>` labels.

## Changing a skill

The flow, the case format and what each case kind checks are in `evals/README.md`.

- After editing a skill, run `scripts/check-skills`. It's free, and the pre-commit hook runs it too.
- The skills are installed globally. On main, each commit copies `.claude/skills/`, minus each `evals/` folder, into `~/.claude/skills/` (`scripts/sync-skills`). A personal skill outranks a project skill of the same name, so that copy is what runs, in this repo too. To try an uncommitted edit by hand, run `scripts/sync-skills --worktree`, then `scripts/sync-skills` to go back to the committed version. The evals read this repo's folder, so they don't need it.
- `scripts/eval-skills <skill>` runs that skill's eval cases once each; `--full` runs them three times with a baseline. Both make billed model runs, so run them when the user asks or agrees. Use `--full` before committing a change to what a skill does.
- A skill counts as implemented when it has at least three eval cases, covering the trigger, first-turn, mid-state and artifact kinds.
- A bug found in a skill gets a regression case in the same change as its fix.
- A bug is something a run showed. A gap found only by reading a skill gets one run against the current text before any fix, when the user agrees to it: a throwaway case in a scratch worktree (`git worktree add --detach <path> HEAD`), about $0.25 on Opus. If the run goes wrong, fix the skill and keep the case as its regression case. If it passes, change nothing and delete the case.
- An eval case is one turn in a sandbox with no web access. What a case cannot reach (a second turn, a page read online, a subagent's real work) gets one real run after the commit that installs the skill. Report what that run showed, and put anything it found in the skill to the user before editing.
- A new skill is wired in outside its own folder too: a routing case that fires it, its name in the grader of `concept-question-fires-nothing`, a row in the README's workflow table, and its entries in this file's layout and "How we work".
- The skills are written for Opus, the model they mainly run on. Evaluate them there: `scripts/eval-skills <skill> --model opus`. Without `--model` the script uses your Claude Code default, which may be another model. A change made for one model gets a baseline: run the affected targets on that model before the edit, and again after.
- Eval runs spend the same usage limit as the session. `eval-skills` stops at the first run that hits it and exits 2: after the reset, rerun the same command with `--resume`, which skips the cases that finished. Don't debug a limit stop.
- The loop skills dispatch their subagents at `high` effort. Go higher only when the user names a level.
- This repository is public. A skill writes repository-relative paths into the files it commits, never absolute ones, and keeps raw tool logs out of git.
- Third-party skills stay out of `.claude/skills/`: everything there is linted, owes eval cases and is installed globally. Install one as a personal skill in `~/.claude/skills/` instead. The auditor, `/scope-problem`'s humanize pass and `/write-concept` load a skill named `humanizer` when the session has one, and this repo's rules win where the two disagree.

## Coaching style

- Skip the fundamentals. Push on trade-offs, failure modes and behavior at scale.
- Rust: explain only async, lifetimes and concurrency subtleties.
- Cite the matching chapter or guide when one exists: Alex Xu's *System Design Interview* (vol. 1–2), *Designing Data-Intensive Applications*, Hello Interview.
- These notes are for long-term learning as well as interview prep: favor understanding why a design works over memorizing patterns.

## Commit messages

Checked by `.githooks/commit-msg` (each clone enables it once: `git config core.hooksPath .githooks`):

```
MM/DD/YY: <slug>: <what changed>      first line, max 72 chars; "<slug>: " only when it's about one system

Co-Authored-By: Name <email>          optional: a blank line, then only Key: value lines
```

- The date is today's local date (`date +%m/%d/%y`). It's checked as a format only.
- Keep messages concise. No free-text body.
- Commit only when the user asks.

## Rust

- The workspace root is `src/Cargo.toml`; `/implement-design` creates it with the first crate. Rust may not be installed yet, so check `cargo --version` first.
- Run cargo from `src/`: `cargo test -p <slug>`, `cargo clippy --all-targets`, `cargo fmt`.

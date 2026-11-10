# Evals

Tests for the skills in `.claude/skills/`. They answer one question: does the skill still do what it says after an edit?

## Run them

```sh
scripts/check-skills                         # lint: free, about a second
scripts/eval-skills scope-problem            # one skill's cases, one run each
scripts/eval-skills routing                  # which skill fires
scripts/eval-skills                          # everything
scripts/eval-skills scope-problem --full     # three runs per case, plus a baseline without the skill
scripts/eval-skills --case 'resumes-*' --model sonnet --max-cost 3
```

`check-skills` runs on its own in the pre-commit hook whenever a commit touches a skill, a case or these scripts. An error blocks the commit; a warning doesn't.

`eval-skills` makes real, billed model runs on your account through `claude plugin eval`. It prints what each target cost, and `--max-cost` sets a ceiling. It needs Claude Code 2.1.269 or later and git 2.31 or later; if the git first on your PATH is older, the script looks for a newer one in the usual places. Reports land in `evals/results/` beside the cases and are not committed.

## The flow

1. Edit a skill.
2. `scripts/check-skills`.
3. `scripts/eval-skills <skill>` for a quick pass.
4. `scripts/eval-skills <skill> --full` before committing a change to what the skill does.
5. When a case fails, decide which is wrong, the skill or the case, and fix that one.

## What every skill owes

A skill counts as implemented when the lint passes and it has at least three cases, covering these kinds. The lint warns about what is missing.

| Kind (the case's tag) | The question it answers |
|---|---|
| `trigger` | Does the skill fire on its own prompts and stay quiet on its neighbor's? These live in `routing/`. |
| `first-turn` | From a repo with nothing in it, does the skill open the way it says? |
| `mid-state` | From a fixture stage, does it pick up at the right place? |
| `artifact` | Do the files it writes meet its template or spec? |
| `regression` | One per bug found, added when the bug is fixed. Optional until there is a bug. |

## Layout

```
evals/
├── fixtures/                 one small sample system, frozen at each stage
│   ├── place.sh              copies a stage into a run's working folder
│   ├── empty/                the bare repo: README.md, CLAUDE.md, no systems
│   ├── mid-sweep/            scoping in progress, with one planted conflict
│   └── scoped/               requirement.md signed off
└── routing/                  a wrapper that loads every skill together
    ├── skills -> ../../.claude/skills
    └── evals/<case>/         which-skill-fires cases
.claude/skills/<skill>/evals/
├── <case>/                   that skill's cases
└── lint.py                   optional: the skill's own checks, run by check-skills
```

The skills form a pipeline, so each skill's cases start from the stage before it: `/new-design` from `scoped/`, and later `/review-design` from a drafted stage and `/implement-design` from a reviewed one. Add a stage when the first case needs it. Every stage is laid over `empty/`, so it holds only what differs.

## A case

One folder, named for what it checks (`resumes-at-conflict-sweep`):

```
prompt.md          run settings, then the prompt exactly as you would type it
case.yaml          the name, what a good result looks like, and the scaffold script
place-files.sh     one line: which fixture stage to start from
graders/NN-….md    one checkable statement per file
```

`prompt.md` starts with a settings block. Only these four keys are allowed:

```markdown
---
tags: [mid-state, regression]
max_turns: 20
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Skill, Write, Edit, Bash]
---

/scope-problem rate-limiter
```

A grader is one statement a judge model can mark true or false from the run's transcript:

```markdown
---
type: llm
focus: trace
weight: 1
---

The response tells the user that scoping resumed at phase 6, step 1, the conflict sweep.
```

To add a case, copy the nearest existing one and change the prompt, the stage and the graders. Word a grader about the result ("requirement.md has a `Status: draft` line"), not about how the skill got there.

## Things the runner does that are easy to trip on

- **One prompt, no replies.** A case tests one turn. The skill asks its questions and the run ends, so a case cannot play out an interview.
- **A run sees only the fixture.** It starts in a temporary folder holding what `place-files.sh` copied, not this repo. Anything the skill reads (`README.md`, `CLAUDE.md`) has to be in the stage.
- **The runner picks its own model.** It doesn't read your Claude Code settings, so `eval-skills` passes your default model for it.
- **`/skill-name` prompts don't count as a Skill call.** A prompt that starts with the slash command loads the skill directly. Use the `tool_used: Skill` grader only in routing cases, where the prompt is plain language.
- **Routing needs the wrapper.** The runner takes one skill folder at a time, so `routing/` exists to load all of them together. In its runs the skills are named `system-design-skills:<skill>`.
- **The judge is a small model voting three times.** A statement that needs judgment can flip between runs. Prefer statements with a plain yes or no, and use `--full` when a result looks marginal.

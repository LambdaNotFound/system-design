# Implementation spec

## Contents

- The brief: what the loop builds from
- Scope: what one crate holds
- `impl.md`: the plan and the loop's log
- The crate: layout and conventions
- The two stages: tests first, then the logic
- Checks: the commands, the record, and what passes

This file is the contract for what `/implement-design` produces. The
coding expert shapes the plan and the crate to it, the bar-raiser
reviews against it, and the lead follows its Checks section. `<dir>`
is the loop directory and `<slug>` the crate's name. Dates are
`MM/DD/YY`.

## The brief

The loop builds from documents it reads where they are. Nothing is
copied, and no loop agent writes any of them. `impl-run.md`'s `Brief:`
line names the paths.

| The slug is | The brief | `<dir>` |
| --- | --- | --- |
| A system or piece routed `design` | `rfc.md`, and `requirement.md` with its `## Amendment` sections | `design-doc/<slug>/` |
| A piece routed `code` | `requirement.md` with its amendments | `design-doc/<slug>/` |
| A concept | `concepts/<slug>.md` | `concepts/<slug>/` |

- The requirement doc says what must hold, by row ID. The RFC says
  how: its component sections, sketches and Work Breakdown. Where the
  two disagree, the requirement doc wins, the latest amendment first,
  and the disagreement is a brief correction to report.
- A concept note has no rows. Its How it works and Trade-offs and
  variants sections are the requirements, cited by heading.
- `design.md`, `review.md` and `scoping.md` may be read for evidence,
  residual risks and open questions. They bind nothing.

## Scope

A crate implements the core mechanism of its design, not the system.

- One to three mechanisms, the ones where the interesting logic
  lives, taken from the RFC's Work Breakdown when there is one. A
  plan that needs more is too large for one crate: propose a split
  into pieces.
- In-process. No real network, store or infrastructure, unless the
  network behavior is the point. A dependency the design names (a
  store, a queue, a clock) is a trait in the crate with an in-memory
  implementation, so a test can make it slow, fail or reorder.
- Time is injected: passed in as a value, or read through a trait. No
  test sleeps or reads the wall clock.
- Std only by default. A dependency is added only when the mechanism
  needs it (`tokio` for async I/O), and the plan says why.
- No `unsafe`, unless the mechanism is about it.

## `impl.md`

The coding expert alone writes it. It holds the plan, which the user
approves before any code exists, and then the loop's log.

```markdown
Covers: <N> amendments, <M> scope lines

# Implementation: <title>

## Mechanisms
## Types
## Test plan
## Not proven here
## Dependencies
## Checks
## Follow-ups
## Revision log
## Objection responses
```

- **Line 1** counts what the plan was written against: the
  `## Amendment` sections in the requirement doc (0 when the brief has
  none) and the `Scope:` lines in `impl-run.md`. The lead's resume
  compares it with the files, so keep its wording exactly.
- **Mechanisms.** Each one with the Work Breakdown task or note
  section it comes from, the rows it serves, and one line on why the
  logic lives there.
- **Types.** The public types and signatures, as a sketch in a `rust`
  block, in the brief's vocabulary. Where the RFC sketched an
  interface, start from it, and say where the crate departs and why.
- **Test plan.** A `| Test | Proves | Kind |` table. `Proves` is a row
  ID or a note heading. `Kind` is behavior, boundary, concurrency or
  failure. Every row the chosen mechanisms serve has a test here or a
  line under Not proven here.
- **Not proven here.** The rows an in-process crate cannot prove, such
  as a latency percentile at production load, each with the reason.
  `none` when there are none.
- **Dependencies.** `none`, or each crate with the reason.
- **Checks.** A fenced `sh` block, one runnable command per line, run
  from `src/`. The default is the three under Checks below.
- **Follow-ups.** Work left out on purpose: a deferred objection, a
  mechanism this crate leaves out. A crate gets one loop, so these
  are for the user to build or to ask for directly.
- **Revision log.** One keyed line per author pass, naming the files
  it touched: `T0:` the first tests pass, `T<N>:` the response to
  tests round N, `R0:` the first implementation, `R<N>:` the response
  to review round N. A pass that fixes failed checks adds to the entry
  of the pass it fixes. The lead's resume reads the keys, so an
  unkeyed entry breaks it.
- **Objection responses.** One line per objection ID: accepted with
  the files and lines that resolve it, rebutted with the evidence, or
  deferred with the reason and a Follow-ups line.

## The crate

- **Workspace.** The root is `src/Cargo.toml`. When it does not exist,
  create it:

  ```toml
  [workspace]
  resolver = "3"
  members = []

  [workspace.package]
  edition = "2024"
  ```

  Then, from `src/`, run `cargo new --lib <slug>`. Check that `<slug>`
  was added to `members`, and that the crate's `Cargo.toml` has
  `edition.workspace = true`.
- **Layout.** The types and the logic are in `src/<slug>/src/`. The
  tests the plan lists are integration tests in `src/<slug>/tests/`,
  so they reach the crate only through its public API and the
  approved set is one folder.
- **Doc comments** state each public item's contract and name where
  it comes from: `/// See design-doc/<slug>/rfc.md, Retry Scheduler.`,
  `/// See design-doc/<slug>/requirement.md, FR2.`, or
  `/// See concepts/<slug>.md, How it works.`
- **Tests** pin the behavior the brief promises, the edge cases it
  calls out included: boundaries, concurrency, failure handling. Each
  one names what it proves in its doc comment (`/// Proves FR2.`), and
  asserts behavior a caller can see, never private state.
- **Concurrency tests** use real threads (`std::thread::scope`) and a
  barrier or a channel to force the interleaving they are about. A
  test that can only fail by luck proves nothing.
- **Comments** explain why, never what. None narrates the code or
  justifies a change to the reviewer.

## The two stages

**Tests stage.** The crate compiles, and nothing in it works yet.

- Every body that holds logic is `todo!()`. A constructor or a plain
  accessor may be real when no test can be written without it; the
  `T0:` entry names each one.
- The tests are complete: every line of the Test plan exists and
  fails at a `todo!()`.
- The bar-raiser's tests review approves the test set. From then on
  the files in `src/<slug>/tests/` are frozen.

**Implementation stage.** The `todo!()`s are filled in.

- A frozen test is changed or removed only when it is wrong, and the
  `R<N>:` entry names the test and the reason. The bar-raiser checks
  every such change.
- New tests are welcome anywhere: a unit test beside a private
  helper, another integration test.
- No `todo!()` and no `unimplemented!()` is left.

## Checks

The lead runs the `## Checks` block once after every author pass,
from `src/`, each command as written. The default block:

```sh
cargo fmt -p <slug> -- --check
cargo clippy -p <slug> --all-targets -- -D warnings
cargo test -p <slug> --no-fail-fast
```

In the tests stage the lead leaves `-- -D warnings` off the clippy
line: a stubbed body leaves its parameters unused, and those warnings
are expected. `--no-fail-fast` keeps a failing unit test from hiding
the integration tests.

Each command's output goes to `<dir>/impl-checks/<label>-<k>.log`,
where `<label>` is the pass's revision-log key (`T0`, `R1`) and `<k>`
the command's number. The record is `<dir>/impl-checks/<label>.md`:

```markdown
# Checks <label>

Stage: tests
Date: <MM/DD/YY>

| # | Command | Exit | Seconds | Log |
| --- | --- | --- | --- | --- |
| 1 | cargo fmt -p <slug> -- --check | 0 | 1 | impl-checks/<label>-1.log |

Tests: <n> run, <n> passed, <n> failed, <n> failed at todo
Passed against stubs: <test names, or none>
Result: pass
```

`Stage:` is `tests` or `implementation`. `Passed against stubs:` is
written in the tests stage only. A second run for the same pass, after
a fix, replaces the record and its logs. The record is committed with
the loop's other files. The logs are not: cargo prints absolute paths
in them, so `.gitignore` keeps `impl-checks/*.log` out of the
repository.

What passes:

- **Tests stage.** The fmt and clippy commands exit 0. The test
  command compiled, so its failures are test failures and not build
  errors. Every failed test failed at a `todo!()`: its panic message
  starts `not yet implemented`. A test that passed against stubs does
  not fail the checks; it is listed for the reviewer.
- **Implementation stage.** Every command exits 0.

`Result:` is `pass` or `fail`. On `fail`, the lead sends the record's
path back to the author once, then runs the checks again.

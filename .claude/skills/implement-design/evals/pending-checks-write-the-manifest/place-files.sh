#!/usr/bin/env bash
# Fixture stage for this case (see evals/fixtures/place.sh), with R0's check record removed.
set -eu
"$(dirname "$0")/../../../../../evals/fixtures/place.sh" scoped loop-approved loop-closed impl-mapped impl-approved
rm design-doc/rate-limiter/impl-checks/R0.md

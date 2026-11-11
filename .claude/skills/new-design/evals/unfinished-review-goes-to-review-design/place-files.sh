#!/usr/bin/env bash
# Fixture stage for this case (see evals/fixtures/place.sh).
set -eu
exec "$(dirname "$0")/../../../../../evals/fixtures/place.sh" scoped loop-approved loop-closed loop-review-unfinished

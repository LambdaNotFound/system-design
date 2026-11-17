#!/usr/bin/env bash
# Fixture stage for this case (see evals/fixtures/place.sh), plus one open brief correction.
set -eu
"$(dirname "$0")/../../../../../evals/fixtures/place.sh" scoped loop-approved loop-closed impl-mapped
printf '%s\n' 'C1 | 11/12/26 | map | FR2: Retry-After is the seconds until the client may retry | under the sliding window the estimate can fall below the limit before the minute ends, so the seconds to the next minute overstate the wait' >> design-doc/rate-limiter/impl-run.md

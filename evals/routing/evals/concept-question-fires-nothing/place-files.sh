#!/usr/bin/env bash
# Copies the fixture this case starts from into the run's working folder.
set -eu
exec "$(dirname "$0")/../../../../evals/fixtures/place.sh"

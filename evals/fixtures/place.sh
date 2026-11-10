#!/usr/bin/env bash
# Copies a fixture stage into the current directory, which is the eval run's
# working folder. Every stage is laid over `empty`, the bare repo skeleton.
#   place.sh            the skeleton only
#   place.sh scoped     the skeleton, then the scoped stage
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
for stage in empty "$@"; do
  [ -d "$here/$stage" ] || { echo "place.sh: no fixture stage '$stage' in $here" >&2; exit 1; }
  cp -R "$here/$stage/." .
done

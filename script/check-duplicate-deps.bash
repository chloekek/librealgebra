#!/usr/bin/env bash

# This script checks that there are no duplicate dependencies
# that we do not expect (according to expected-duplicate-deps.txt).

set -o errexit

diff --unified <(cargo tree --duplicates) script/expected-duplicate-deps.txt

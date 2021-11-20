#!/usr/bin/env bash

set -o errexit
set -o xtrace

script/check-duplicate-deps.bash
cargo build
cargo test
cargo doc

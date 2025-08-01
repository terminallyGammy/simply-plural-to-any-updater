#!/bin/bash

set -euo pipefail

rustfmt --edition 2021 src/**.rs

# NOTE! Sync -W args with vscode settings when changd
cargo clippy --allow-dirty --fix -- \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
# NOTE! Sync -W args this with vscode settings when changd

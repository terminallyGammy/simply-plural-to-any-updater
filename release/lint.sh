#!/bin/bash

set -euo pipefail

rustfmt --edition 2021 src/**.rs

cargo clippy --allow-dirty --fix -- \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \

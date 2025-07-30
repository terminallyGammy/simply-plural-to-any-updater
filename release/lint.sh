#!/bin/bash

set -euo pipefail

cargo clippy --allow-dirty --fix -- \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    

rustfmt --edition 2021 src/**.rs

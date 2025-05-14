#!/bin/bash

set -euo pipefail

echo "Build..."
cargo build --bin sps_status --release
echo "Done."

echo "Run:"
set -a; source defaults.env; set +a
set -a; source dev.vrcupdater.env; set +a
./target/release/sps_status


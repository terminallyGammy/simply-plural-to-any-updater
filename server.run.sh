#!/bin/bash

set -euo pipefail

echo "Build..."
cargo build --release
echo "Done."

echo "Run:"
set -a; source defaults.env; set +a
set -a; source dev.server.env; set +a
./target/release/sps_status


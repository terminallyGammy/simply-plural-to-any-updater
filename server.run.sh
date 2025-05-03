#!/bin/bash

set -euo pipefail

echo "Build..."
cargo build --release
echo "Done."

echo "Run:"
set -a; source .env; set +a
export SERVE_API=true
./target/release/sps_status


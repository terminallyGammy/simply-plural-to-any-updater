#!/bin/bash

set -euo pipefail

echo "Build..."
cargo build --bin sps_status --release
echo "Done."

echo "Run:"
rm -f vrcupdater.env || true
cp dev.vrcupdater.env vrcupdater.env # uncomment this line to test absence of env file
./target/release/sps_status


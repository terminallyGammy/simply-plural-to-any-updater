#!/bin/bash

set -euo pipefail

echo "Build..."
cargo tauri build
echo "Done."

echo "Run:"
rm -f vrcupdater.env || true
cp dev.sp2any.env vrcupdater.env
cargo tauri dev

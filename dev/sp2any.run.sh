#!/bin/bash

set -euo pipefail

echo "Run:"
rm -f vrcupdater.env || true
cp dev/sp2any.env vrcupdater.env
./dev/tauri-dev.sh

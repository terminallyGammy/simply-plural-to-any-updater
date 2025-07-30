#/bin/bash

set -euo pipefail

(cd frontend && npm ci)

cargo tauri build --verbose "$@"

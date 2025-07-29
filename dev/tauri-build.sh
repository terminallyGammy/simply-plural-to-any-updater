#/bin/bash

set -euo pipefail

cargo tauri build --verbose "$@"

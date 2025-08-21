#!/bin/bash

set -euo pipefail

echo "===== CLOC ===="
cloc --exclude-dir=node_modules,target --exclude-ext=json .

echo "===== Rust lines of code per file ==="
ls src/*/* | xargs wc -l

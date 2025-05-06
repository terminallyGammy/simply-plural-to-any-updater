#!/bin/bash

set -euo pipefail

./stop-services.sh || true

# cargo must be installed
cargo build --release

docker compose build
docker compose up -d


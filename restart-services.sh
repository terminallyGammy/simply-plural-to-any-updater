#!/bin/bash

set -euo pipefail

./stop-services.sh || true

# cargo must be installed
cargo build --release

docker compose -f server.docker-compose.yml build
docker compose -f server.docker-compose.yml up -d


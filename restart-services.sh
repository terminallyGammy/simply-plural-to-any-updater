#!/bin/bash

set -euo pipefail

./stop-services.sh || true

cargo build --release
docker compose build
docker compose up -d


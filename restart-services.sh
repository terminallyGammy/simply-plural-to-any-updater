#!/bin/bash

set -euo pipefail

./stop-services.sh || true

docker compose build
docker compose up -d


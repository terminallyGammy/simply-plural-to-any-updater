#!/bin/bash

set -euo pipefail

./stop-services.sh || true

docker compose -f server.docker-compose.yml build
docker compose -f server.docker-compose.yml up -d


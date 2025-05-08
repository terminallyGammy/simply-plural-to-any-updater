#!/bin/bash

set -euo pipefail

docker compose -f dev.pwsh.docker-compose.yml down || true

docker compose -f dev.pwsh.docker-compose.yml build
docker compose -f dev.pwsh.docker-compose.yml up


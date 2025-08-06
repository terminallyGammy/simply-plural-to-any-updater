#!/bin/bash

set -euo pipefail

./stop-services.sh || true

LATEST_RELEASE_URL="$( curl -w "%{url_effective}\n" -o /dev/null -I -L -s -S https://github.com/GollyTicker/simply-plural-to-any-updater/releases/latest )"

echo "Using latest release from: $LATEST_RELEASE_URL"

DOWNLOAD_URL="$( echo "${LATEST_RELEASE_URL/tag/download}"/SP2Any-Linux )"

echo "Downloading binary from to $DOWNLOAD_URL to target/SP2Any-Linux"

curl -L -s -o target/SP2Any-Linux "$DOWNLOAD_URL"

docker compose -f server.docker-compose.yml build
docker compose -f server.docker-compose.yml up -d


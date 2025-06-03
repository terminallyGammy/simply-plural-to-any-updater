#!/bin/bash

set -euo pipefail

# unit tests
cargo test --bin sps_status


# integration test
# cargo build --bin sps_status --release

# todo. add integration tests with new test system here.
# 1. set system fronts
# 2. start webserver
# 3. check correct system fronts on website
# 4. change fronts
# 5. check new fronts on website

# 1. set system fronts
# 2. start VRC updater
# 3. check vrc status correct after a few seconds of startup
# 4. change fronts
# 5. check vrc status correct after 60 seconds

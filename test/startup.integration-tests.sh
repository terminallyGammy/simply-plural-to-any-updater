#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$VRCHAT_COOKIE" != "" ]]

[[ "$DISCORD_TOKEN" != "" ]]

FUNCTIONAL_DISCORD_TOKEN="$DISCORD_TOKEN"
FUNCTIONAL_VRCHAT_USERNAME="$VRCHAT_USERNAME"
FUNCTIONAL_SPS_API_TOKEN="$SPS_API_TOKEN"

ENABLE_DISCORD=true
ENABLE_VRCHAT=true

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_updater
    ./release/cargo-build.sh


    setup_sp_rest_failure
    start_updater
    check_updater_failure
    check_updater_loop_continues
    reset_changed_variables


    stop_updater
    setup_vrchat_only
    start_updater
    check_updater_has_no_errors
    check_updater_loop_continues
    reset_changed_variables


    stop_updater
    setup_discord_only
    start_updater
    check_updater_has_no_errors
    check_updater_loop_continues
    reset_changed_variables


    stop_updater
    setup_vrchat_misconfigured
    start_updater
    check_updater_failure
    check_updater_loop_continues
    reset_changed_variables


    stop_updater
    clear_all_fronts
    echo "✅✅✅ Updater Integration Test ✅✅✅"
}


check_updater_has_no_errors() {
    echo "check_updater_has_no_errors"
    [[ "$( docker logs sp2any-webserver 2>&1 | grep "Error" | wc -l )" == "0" ]]
}

check_updater_loop_continues() {
    echo "check_updater_loop_continues"
    docker logs sp2any-webserver 2>&1 | grep -q "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."
}

check_updater_failure() {
    echo "check_updater_failure"
    docker logs sp2any-webserver 2>&1 | grep -q "Error"
}


setup_sp_rest_failure() {
    echo "setup_sp_rest_failure"
    SPS_API_TOKEN="invalid"
}

setup_vrchat_only() {
    echo "setup_vrchat_only"
    DISCORD_TOKEN="invalid"
    ENABLE_DISCORD=false
}

setup_discord_only() {
    echo "setup_discord_only"
    VRCHAT_USERNAME="invalid"
    ENABLE_VRCHAT=false
}

setup_vrchat_misconfigured() {
    echo "setup_vrchat_misconfigured"
    VRCHAT_USERNAME="invalid"
    # VRCHAT enabled!
}


reset_changed_variables() {
    echo "reset_changed_variables"
    DISCORD_TOKEN="$FUNCTIONAL_DISCORD_TOKEN"
    VRCHAT_USERNAME="$FUNCTIONAL_VRCHAT_USERNAME"
    SPS_API_TOKEN="$FUNCTIONAL_SPS_API_TOKEN"
    ENABLE_DISCORD=true
    ENABLE_VRCHAT=true
}


export BASE_URL="http://localhost:8000"

start_updater() {
    echo "start_updater"
    ./docker/local.start.sh > /dev/null 2>&1

    setup_test_user

    # ensure the automatic restart of updaters happens during startup
    docker restart sp2any-webserver

    await sp2any-webserver "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

    echo "Started startup-test."
}

stop_updater() {
    echo "stop_updater"
    ./docker/local.stop.sh > /dev/null 2>&1
    echo "Stopped startup-test."
}
trap stop_updater EXIT

main

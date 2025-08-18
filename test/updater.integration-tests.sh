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


    set_system_fronts_set "A"
    start_updater
    check_system_fronts_set "A"
    set_system_fronts_set "B"
    sleep "$SECONDS_BETWEEN_UPDATES"s
    check_system_fronts_set "B"


    stop_updater
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


check_system_fronts_set() {
    SET="$1"

    if [[ "$SET" == "A" ]]; then
        check_vrc_status_string_equals "F˸Ann‚Bor‚Dae‚Cst"
        check_discord_status_string_equals "F: Annalea 💖 A., Borgn B., Daenssa 📶 D., Cstm First"
    elif [[ "$SET" == "B" ]]; then
        check_vrc_status_string_equals "F˸ tešt t․"
        check_discord_status_string_equals "F: tešt ▶️ t."
    else
        return 1
    fi
}


check_vrc_status_string_equals() {
    EXPECTED="$1"

    RESPONSE="$(curl -s "https://api.vrchat.cloud/api/1/auth/user" \
        --cookie "$VRCHAT_COOKIE" \
        -u "$VRCHAT_USERNAME:$VRCHAT_PASSWORD" \
        -H "User-Agent: SP2Any/0.1.0 does-not-exist-792374@gmail.com"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .statusDescription)"

    echo "VRC Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}

check_discord_status_string_equals() {
    EXPECTED="$1"

    RESPONSE="$(curl -s \
        "https://discord.com/api/v10/users/@me/settings" \
        -H "Authorization: $DISCORD_TOKEN"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .custom_status.text )"

    echo "Discord Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}

check_updater_has_no_errors() {
    [[ "$( docker logs sp2any-webserver 2>&1 | grep "Error" | wc -l )" == "0" ]]
}

check_updater_loop_continues() {
    docker logs sp2any-webserver 2>&1 | grep -q "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."
}

check_updater_failure() {
    docker logs sp2any-webserver 2>&1 | grep -q "Error"
}



setup_sp_rest_failure() {
    SPS_API_TOKEN="invalid"
}

setup_vrchat_only() {
    DISCORD_TOKEN="invalid"
    ENABLE_DISCORD=false
}

setup_discord_only() {
    VRCHAT_USERNAME="invalid"
    ENABLE_VRCHAT=false
}

setup_vrchat_misconfigured() {
    VRCHAT_USERNAME="invalid"
    # VRCHAT enabled!
}


reset_changed_variables() {
    DISCORD_TOKEN="$FUNCTIONAL_DISCORD_TOKEN"
    VRCHAT_USERNAME="$FUNCTIONAL_VRCHAT_USERNAME"
    SPS_API_TOKEN="$FUNCTIONAL_SPS_API_TOKEN"
    ENABLE_DISCORD=true
    ENABLE_VRCHAT=true
}


export BASE_URL="http://localhost:8000"

start_updater() {
    ./docker/local.start.sh

    setup_test_user

    echo "Started Updater."
}

stop_updater() {
    ./docker/local.stop.sh
    echo "Stopped Updater."
}
trap stop_updater EXIT

main

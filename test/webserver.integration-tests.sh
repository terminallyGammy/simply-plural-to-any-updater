#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

source ./test/plural_system_to_test.sh

main() {
    clear_all_fronts

    set_initial_system_fronts

    stop_webserver

    start_webserver

    check_initial_system_fronts

    clear_all_fronts

    set_second_system_fronts

    check_second_system_fronts

    stop_webserver

    clear_all_fronts

    echo "✅✅✅ Webserver Integration Test ✅✅✅"
}


set_initial_system_fronts() {
    set_to_front "$ANNALEA_ID"
    set_to_front "$BORGNEN_ID"
    set_to_front "$DAENSSA_ID"
}

check_initial_system_fronts() {
    HTML="$(curl -s "$WEBSERVER_FRONTING_URL")"
    grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
    grep '<div><img src="https://example.com/a" /><p>Annalea 💖 A.</p></div>' <<< "$HTML"
    grep '<div><img src="https://example.com/b" /><p>Borgnen 👍 B.</p></div>' <<< "$HTML"
    grep '<div><img src="" /><p>Daenssa 📶 D.</p></div>' <<< "$HTML"
    [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "3" ]]
}

set_second_system_fronts() {
    set_to_front "$TEST_MEMBER_ID"
}

check_second_system_fronts() {
    HTML="$(curl -s "$WEBSERVER_FRONTING_URL")"
    grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
    grep '<div><img src="" /><p>tešt ▶️ t.</p></div>' <<< "$HTML"
    [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "1" ]]
}

WEBSERVER_FRONTING_URL="http://0.0.0.0:8000/fronting"

start_webserver() {
    cargo build --bin sps_status --release
    
    set -a; source defaults.env; set +a
    export SPS_API_TOKEN
    export SERVE_API=true
    export SYSTEM_PUBLIC_NAME="SP-Updater-Test"

    ./target/release/sps_status &

    sleep 1s
}

stop_webserver() {
    pkill -f sps_status || true
}

main

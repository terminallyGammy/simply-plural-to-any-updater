# integration test
# cargo build --bin sps_status --release

#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

source ./test/plural_system_to_test.sh

main() {
    stop_vrc_updater

    set_system_fronts_set "A"
    
    start_vrc_updater

    # check_system_fronts_set "A"
    
    # set_system_fronts_set "B"

    # sleep 60s

    # check_system_fronts_set "B"

    stop_vrc_updater

    clear_all_fronts

    # echo "✅✅✅ VRC Updater Integration Test ✅✅✅"
}

check_system_fronts_set() {
    SET="$1"

    HTML="$(curl -s "$WEBSERVER_FRONTING_URL")"

    if [[ "$SET" == "A" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="https://example.com/a" /><p>Annalea 💖 A.</p></div>' <<< "$HTML"
        grep '<div><img src="https://example.com/b" /><p>Borgnen 👍 B.</p></div>' <<< "$HTML"
        grep '<div><img src="" /><p>Daenssa 📶 D.</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "3" ]]
    elif [[ "$SET" == "B" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="" /><p>tešt ▶️ t.</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "1" ]]
    else
        return 1
    fi
}


start_vrc_updater() {
    cargo build --bin sps_status --release

    rm -rf vrcupdater.env || true
    
    export SPS_API_TOKEN
    export VRCHAT_USERNAME
    export VRCHAT_PASSWORD
    export VRCHAT_COOKIE

    echo "SPS_API_TOKEN=\"$SPS_API_TOKEN\"" >> vrcupdater.env
    echo "VRCHAT_USERNAME=\"$VRCHAT_USERNAME\"" >> vrcupdater.env
    echo "VRCHAT_PASSWORD=\"$VRCHAT_PASSWORD\"" >> vrcupdater.env
    echo "VRCHAT_COOKIE=\"$VRCHAT_COOKIE\"" >> vrcupdater.env

    (./target/release/sps_status 2>&1 | sed 's/^/VRC Updater | /g' ) & 

    sleep 5s

    echo "Started VRC Updater."
}

stop_vrc_updater() {
    pkill -f sps_status || true
    echo "Stopped VRC Updater."
}

main

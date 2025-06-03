#!/bin/bash


export ANNALEA_ID="683f23e79aa188caf3000000"
export BORGNEN_ID="683f23f49aa189caf3000000"
export CLENNTRO_ID="683f24009aa18acaf3000000"
export DAENSSA_ID="683f24179aa18bcaf3000000"
export TEST_MEMBER_ID="683f243e9aa18ccaf3000000"

set_to_front() {
    MEMBER_ID="$1"
    FRONT_ID="$(openssl rand -hex 12)" # produces valid 24 hexdec digits
    curl -s --fail-with-body -L "https://api.apparyllis.com/v1/frontHistory/$FRONT_ID" \
        -H 'Content-Type: application/json' \
        -H "Authorization: $SPS_API_WRITE_TOKEN" \
        -d "{
            \"customStatus\": \"\",
            \"custom\": false,
            \"live\": true,
            \"startTime\": 0,
            \"member\": \"$MEMBER_ID\"
        }" > /dev/null
    echo "Set member $MEMBER_ID to front (id: $FRONT_ID)."
}

clear_all_fronts() {
    FRONTER_IDS="$(
        curl --silent --fail-with-body \
            -L 'https://api.apparyllis.com/v1/fronters/' \
            -H "Authorization: $SPS_API_WRITE_TOKEN" |
            jq -r '.[].id'
    )"

    if [[ "$FRONTER_IDS" == "" ]]; then
        return 0
    fi

    while read fronter_id; do
        echo "Clearing front (id=$fronter_id)"
        curl -L -X PATCH "https://api.apparyllis.com/v1/frontHistory/$fronter_id" \
            -H 'Content-Type: application/json' \
            -H "Authorization: $SPS_API_WRITE_TOKEN" \
            -d '{
                "live": false,
                "startTime": 0,
                "endTime": 15,
                "customStatus": "",
                "custom": false
            }'
    done <<< "$FRONTER_IDS"
}

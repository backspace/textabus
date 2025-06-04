#!/bin/bash

BASE_URL="https://api.winnipegtransit.com/v4"
DATE_PARAM=$(date +%Y-%m-%d)

fetch_and_format_api_response() {
  local endpoint=$1
  local output_file=$2
  local extra_query_parameters=${3:-""}
  
  curl "${BASE_URL}/${endpoint}?api-key=${API_KEY}&usage=short&effective-on=${DATE_PARAM}${extra_query_parameters}" | jq "." > "${output_file}"
}

fetch_and_format_api_response "locations:245%20smith.json" "stops/locations-address.json"
fetch_and_format_api_response "locations:portage%20%40%20main.json" "stops/locations-intersection.json"
fetch_and_format_api_response "locations:assiniboia%20downs.json" "stops/locations-no-stops.json"
fetch_and_format_api_response "locations:jortleby.json" "stops/locations-none.json"
fetch_and_format_api_response "locations:union%20station.json" "stops/locations.json"

fetch_and_format_api_response "stops.json" "stops/stops.json" "&x=634017&y=5527953&distance=500"
fetch_and_format_api_response "stops.json" "stops/stops-none.json" "&x=734017&y=5527953&distance=500"

STOP_NUMBERS=(10625 10641 11052 11010 10642 10901 10902 10624 10830 10590 10907 10639 10939 10589 11051 10651 11024 10620 10675 10803 10804 10591 10158 10588 10672 10652 10157)

for STOP_NUMBER in "${STOP_NUMBERS[@]}"
do
  fetch_and_format_api_response "routes.json" "stops/routes/stop_${STOP_NUMBER}.json" "&stop=${STOP_NUMBER}"
done
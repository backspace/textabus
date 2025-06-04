#!/bin/bash

curl "https://api.winnipegtransit.com/v4/locations:245%20smith.json?api-key=${API_KEY}&usage=short&effective-on=$(date +%Y-%m-%d)" | jq "." > "locations-address.json"

curl "https://api.winnipegtransit.com/v4/locations:portage%20%40%20main.json?api-key=${API_KEY}&usage=short&effective-on=$(date +%Y-%m-%d)" | jq "." > "locations-intersection.json"

curl "https://api.winnipegtransit.com/v4/locations:assiniboia%20downs.json?api-key=${API_KEY}&usage=short&effective-on=$(date +%Y-%m-%d)" | jq "." > "locations-no-stops.json"

curl "https://api.winnipegtransit.com/v4/locations:jortleby.json?api-key=${API_KEY}&usage=short&effective-on=$(date +%Y-%m-%d)" | jq "." > "locations-none.json"

curl "https://api.winnipegtransit.com/v4/locations:union%20station.json?api-key=${API_KEY}&usage=short&effective-on=$(date +%Y-%m-%d)" | jq "." > "locations.json"

STOP_NUMBERS=(10625 10641 11052 11010 10642 10901 10902 10624 10830 10590 10907 10639 10939 10589 11051 10651 11024 10620 10675 10803 10804 10591 10158 10588 10672 10652 10157)

for STOP_NUMBER in "${STOP_NUMBERS[@]}"
do
  curl "https://api.winnipegtransit.com/v4/routes.json?api-key=${API_KEY}&stop=${STOP_NUMBER}&effective-on=$(date +%Y-%m-%d)" | jq "." > "routes/stop_${STOP_NUMBER}.json"
done
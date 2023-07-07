#!/bin/bash

set -euo pipefail

mkdir -p problems

for (( i=1; i<=55; i++ ))
do
    url="https://api.icfpcontest.com/problem?problem_id=$i"

    filename="$i.json"

    json_response=$(curl -s "$url")

    if [ $? -eq 0 ]; then
        echo "Downloaded $filename successfully."
    else
        echo "Failed to download $filename."
    fi

    # Remove the outer quotes from the JSON response
    json_response=$(echo "$json_response" | sed 's/^"//' | sed 's/"$//')

    # Remove the escape characters from the inner JSON
    json_inner=$(echo "$json_response" | jq -r '.Success' | sed 's/\\n/ /g')
    json_minified=$(echo $json_inner | jq -rc)

    echo "$json_minified" > "problems/$filename"
done

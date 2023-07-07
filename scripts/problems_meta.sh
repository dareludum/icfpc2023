#!/bin/bash

if [ $# -eq 0 ]; then
  echo "Folder path is missing."
  echo "Usage: bash script.sh <folder_path>"
  exit 1
fi

folder_path="$1"

if [ ! -d "$folder_path" ]; then
  echo "Invalid folder path: $folder_path"
  exit 1
fi

for file_path in "$folder_path"/*.json; do
  file_name=$(basename "$file_path" .json)

  json_data=$(cat "$file_path")

  attendees=$(echo "$json_data" | jq -r '.attendees | length')
  musicians=$(echo "$json_data" | jq -r '.musicians | length')

  echo "File: $file_name"
  echo "Number of Attendees: $attendees"
  echo "Number of Musicians: $musicians"
  echo
done

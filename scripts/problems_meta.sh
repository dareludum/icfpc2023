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

files=$(ls -1v "$folder_path"/*.json)

for file_path in $files; do
  file_name=$(basename "$file_path" .json)

  json_data=$(cat "$file_path")

  attendees=$(echo "$json_data" | jq -r '.attendees | length')
  musicians=$(echo "$json_data" | jq -r '.musicians | length')
  pillars=$(echo "$json_data" | jq -r '.pillars | length')

  echo "File: $file_name"
  echo "Number of Attendees: $attendees"
  echo "Number of Musicians: $musicians"
  echo "Number of Pillars: $pillars"
  echo
done

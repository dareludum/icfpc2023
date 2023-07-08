#!/bin/sh

set -eo pipefail

API_ROOT=https://api.icfpcontest.com
PROJECT_ROOT=$(dirname "$(realpath "$0")")/..

RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
RESET=$(tput sgr0)

err() {
    local fmt="$1"
    shift
    printf 2>&1 "${RED}${fmt}${RESET}\n" "$@"
}

if [ -z "$ICFPC_TOKEN" ]; then
    ICFPC_TOKEN="$(cat "$PROJECT_ROOT"/.icfpc-token 2>/dev/null || :)"
fi

if [ -z "$ICFPC_TOKEN" ]; then
    err "Missing ICFPC_TOKEN env var or $PROJECT_ROOT/.icfpc-token token file"
    exit 1
fi

icfpc_curl() {
    curl -sSL -H "Authorization: Bearer ${ICFPC_TOKEN}" "$@"
}

submission_payload() {
    jq -sR '{"problem_id": $problem_id, "contents": .}' \
        --argjson problem_id "${1:?missing problem id}"
}

post_submission() {
    icfpc_curl -X POST "${API_ROOT}/submission" \
        -H 'Content-Type: application/json' \
        --data-binary @-
}

query_submission() {
    icfpc_curl -X GET "${API_ROOT}/submission?submission_id=${1:?missing submission id}"
}

submit_solution() {
    local dir="${1:?missing dir}"
    local problem_id="${2:?missing problem id}"
    local solution_path="$dir/${problem_id}_solution.json"
    local meta_path="$dir/${problem_id}_meta.json"

    local score="$(jq .score < "$meta_path")"
    local submission_id="$(submission_payload "$problem_id" < "$solution_path" | post_submission | tr -d '"')"
    printf "Submission id: '%s'\n" "$submission_id"

    if [ -n "$NOWAIT" ]; then
        return
    fi

    if [ -z "$submission_id" ]; then
        err "Empty submission id, bailing out"
        return
    fi

    while true; do
        local submission_status="$(query_submission "$submission_id")"
        local failure_message="$(printf "%s" "$submission_status" | jq .Failure)"
        if [ "$failure_message" != 'null' ]; then
            err "API error: %s" "$failure_message"
            return
        fi

        local submission_score="$(printf "%s" "$submission_status" | jq .Success.submission.score)"
        if [ "$submission_score" = '"Processing"' ]; then
            printf "Processing, awaiting 1sec...\n"
            sleep 1
            continue
        fi

        local failure_message="$(printf "%s" "$submission_score" | jq .Failure)"
        if [ "$failure_message" != 'null' ]; then
            err "submission error: %s" "$failure_message"
            return
        fi
        break
    done

    local remote_score="$(printf "%s" "$submission_score" | jq .Success)"
    printf "local score:  %s\n" "$score"
    printf "remote score: %s\n" "$remote_score"
    if [ "$score" != "$remote_score" ]; then
        err 'SCORE MISMATCH'
        exit 1
    fi
}


if [ "$#" -lt 2 ]; then
    printf 2>&1 "Usage: DIRECTORY [PROBLEM_ID]...\n"
    exit 1
fi

SUBMISSION_DIR="$1"
shift

for problem_id; do
    submit_solution "${SUBMISSION_DIR}" "${problem_id}"
done

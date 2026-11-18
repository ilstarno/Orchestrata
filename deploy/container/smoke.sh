#!/usr/bin/env bash
set -euo pipefail

image=${1:?Usage: bash deploy/container/smoke.sh IMAGE}
container=$(docker run --detach --read-only --cap-drop=ALL \
  --security-opt=no-new-privileges --publish 127.0.0.1::3000 "$image")
trap 'docker rm -f "$container" >/dev/null' EXIT

endpoint=$(docker port "$container" 3000/tcp)
if [[ ! "$endpoint" =~ ^127\.0\.0\.1:[0-9]+$ ]]; then
  printf 'Unexpected published address: %s\n' "$endpoint" >&2
  exit 1
fi

for _ in {1..30}; do
  if response=$(curl --fail --silent --max-time 2 "http://$endpoint/api/v1/health") &&
    [[ "$response" == '{"status":"ok"}' ]]; then
    printf 'API container healthy at %s\n' "$endpoint"
    exit 0
  fi
  sleep 1
done

docker logs "$container" >&2
printf 'API did not answer the health check\n' >&2
exit 1

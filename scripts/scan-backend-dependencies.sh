#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
test -s backend/Cargo.lock || { printf 'Missing backend/Cargo.lock\n' >&2; exit 1; }

if [[ "$(uname -s):$(uname -m)" != Linux:x86_64 ]]; then
    printf 'This scanner script requires Linux x86_64.\n' >&2
    exit 1
fi

# Optional: reuse a verified scanner and a previously downloaded OSV database.
# OSV_SCANNER_BINARY must be the pinned executable below.
# OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY must contain osv-scalibr/crates.io/all.zip.
temp_dir="$(mktemp -d)"
trap 'rm -rf -- "$temp_dir"' EXIT

scanner="${OSV_SCANNER_BINARY:-"$temp_dir/osv-scanner"}"
scanner_sha256=fb7dab19a8bd9bbe5c3189f09d4b7ee1820c9553a23bac202648a56efd2f6ee8
if [[ -z "${OSV_SCANNER_BINARY:-}" ]]; then
    # Google OSV-Scanner v2.7.1 Linux x86_64 release asset.
    curl --proto '=https' --proto-redir '=https' --tlsv1.2 --fail --silent --show-error \
        --location --retry 2 --max-time 120 --header 'Accept: application/octet-stream' \
        https://api.github.com/repos/google/osv-scanner/releases/assets/726165286 \
        --output "$scanner"
fi
printf '%s  %s\n' "$scanner_sha256" "$scanner" | sha256sum --check
if [[ -z "${OSV_SCANNER_BINARY:-}" ]]; then
    chmod 0755 "$scanner"
fi

export OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY="${OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY:-"$temp_dir/cache"}"
database="$OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY/osv-scalibr/crates.io/all.zip"
if [[ "$OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY" == "$temp_dir/cache" ]]; then
    mkdir -p -- "$(dirname -- "$database")"
    curl --proto '=https' --proto-redir '=https' --tlsv1.2 --fail --silent --show-error \
        --location --retry 2 --max-time 120 \
        https://storage.googleapis.com/osv-vulnerabilities/crates.io/all.zip --output "$database"
fi
test -s "$database" || { printf 'Missing offline crates.io OSV database\n' >&2; exit 1; }

"$scanner" scan source --offline --lockfile=backend/Cargo.lock

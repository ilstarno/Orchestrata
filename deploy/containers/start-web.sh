#!/bin/sh
set -eu

upstream=${ORCHESTRATA_API_UPSTREAM:?set ORCHESTRATA_API_UPSTREAM to host:port}
if ! printf '%s\n' "$upstream" | grep -Eq '^[a-zA-Z0-9][a-zA-Z0-9.-]*:[0-9]{1,5}$' ||
    [ "$(printf '%s' "$upstream" | wc -l)" -ne 0 ]; then
    echo 'ORCHESTRATA_API_UPSTREAM must be a DNS name or IPv4 address followed by a port' >&2
    exit 1
fi
port=${upstream##*:}
if [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
    echo 'ORCHESTRATA_API_UPSTREAM port must be in 1..65535' >&2
    exit 1
fi

envsubst '${ORCHESTRATA_API_UPSTREAM}' </etc/nginx/orchestrata.conf.template >/tmp/orchestrata-nginx.conf
exec nginx -c /tmp/orchestrata-nginx.conf -g 'daemon off;'

#!/usr/bin/env bash
# Simple wait-for script for docker-compose services
set -e
host=$1
port=$2
shift 2 || true

until nc -z "$host" "$port"; do
  echo "Waiting for $host:$port..."
  sleep 1
done

echo "$host:$port is available"

# When used as a pure "wait" helper (no extra command), just exit 0.
if [ "$#" -gt 0 ]; then
  exec "$@"
fi

exit 0

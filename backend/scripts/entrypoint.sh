#!/usr/bin/env bash
set -e

# wait for postgres and mongo to be reachable
/usr/local/bin/wait-for-it.sh "${POSTGRES_HOST:-postgres}" "${POSTGRES_PORT:-5432}"
/usr/local/bin/wait-for-it.sh "${MONGODB_HOST:-mongo}" 27017

exec /usr/local/bin/backend

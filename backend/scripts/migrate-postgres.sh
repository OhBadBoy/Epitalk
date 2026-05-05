#!/usr/bin/env bash
set -e
echo "Applying Postgres migrations (scripts/init-postgres.sql)"
docker-compose exec -T postgres psql -U ${POSTGRES_USER:-postgres} -d ${POSTGRES_DB:-chat} -f /scripts/init-postgres.sql || true

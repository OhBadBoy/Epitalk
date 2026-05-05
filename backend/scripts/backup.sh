#!/usr/bin/env bash
echo "Backup MongoDB to ./backups"
mkdir -p backups
docker exec $(docker-compose ps -q mongo) /usr/bin/mongodump --db chat --out /data/backup || true

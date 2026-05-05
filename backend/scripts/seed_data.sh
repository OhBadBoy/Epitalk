#!/usr/bin/env bash
echo "Seeding MongoDB with example data..."
docker-compose exec -T mongo mongo /scripts/init-mongo.js || true

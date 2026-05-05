#!/usr/bin/env bash
set -e
echo "Example: register"
curl -s -X POST -H "Content-Type: application/json" -d '{"email":"user@example.com","username":"user","password":"pass"}' http://localhost:3001/auth/register

echo "\nExample: login"
curl -s -X POST -H "Content-Type: application/json" -d '{"email":"user@example.com","password":"pass"}' http://localhost:3001/auth/login

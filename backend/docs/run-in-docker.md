# Run the project with Docker

1. Copy `.env.example` to `.env` and edit values.
2. Start services:

```bash
docker-compose up --build -d
```

3. Init Postgres (optional):

```bash
docker cp scripts/init-postgres.sql $(docker-compose ps -q postgres):/init-postgres.sql
docker-compose exec postgres psql -U postgres -d chat -f /init-postgres.sql
```

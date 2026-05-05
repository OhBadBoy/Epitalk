# Guide de déploiement

## Vue d'ensemble

Ce guide couvre le déploiement du backend Discord Clone en production.

## Environnements

| Environnement | URL | Trigger |
| ------------- | --- | ------- |
| Development | localhost:3001 | Local |
| Staging | staging.discord-clone.example.com | Push sur `main` |
| Production | api.discord-clone.example.com | Tag `v*.*.*` |

## Prérequis production

### Infrastructure minimale

| Ressource | Specs minimales |
| --------- | --------------- |
| Backend | 2 vCPU, 4 GB RAM |
| PostgreSQL | 2 vCPU, 4 GB RAM, 50 GB SSD |
| MongoDB | 2 vCPU, 4 GB RAM, 100 GB SSD |
| Load Balancer | Nginx ou cloud LB |

### Services requis

- Docker runtime
- PostgreSQL 16+
- MongoDB 7+
- Certificat SSL/TLS
- DNS configuré

## Déploiement avec Docker

### 1. Build de l'image

```bash
# Build local
docker build -t discord-clone-backend:latest ./backend

# Ou utiliser l'image du registry
docker pull ghcr.io/epitechmscpropromo2028/t-jsf-600-par_20/backend:latest
```

### 2. Configuration production

Créer un fichier `.env.production` :

```bash
# Server
HOST=0.0.0.0
PORT=3001
RUST_LOG=info

# Database
DATABASE_URL=postgres://user:password@postgres-host:5432/discord_prod
MONGODB_URI=mongodb://mongo-host:27017/discord_prod

# JWT
JWT_PRIVATE_KEY_PATH=/secrets/jwt_private.pem
JWT_PUBLIC_KEY_PATH=/secrets/jwt_public.pem
JWT_ACCESS_EXPIRY=900
JWT_REFRESH_EXPIRY=604800

# Security
ALLOWED_ORIGINS=https://discord-clone.example.com
```

### 3. Docker Compose production

```yaml
version: '3.8'

services:
  backend:
    image: ghcr.io/epitechmscpropromo2028/t-jsf-600-par_20/backend:latest
    restart: always
    ports:
      - "3001:3001"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - MONGODB_URI=${MONGODB_URI}
      - RUST_LOG=info
    volumes:
      - ./secrets:/secrets:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G

  postgres:
    image: postgres:16-alpine
    restart: always
    environment:
      POSTGRES_DB: discord_prod
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  mongo:
    image: mongo:7
    restart: always
    volumes:
      - mongo_data:/data/db
    healthcheck:
      test: ["CMD", "mongosh", "--eval", "db.adminCommand('ping')"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  mongo_data:
```

### 4. Lancer en production

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Déploiement avec CI/CD

### GitHub Actions

Le workflow `.github/workflows/deploy.yml` gère le déploiement automatique.

**Triggers** :
- Push sur `main` → Staging
- Tag `v*.*.*` → Production

**Étapes** :
1. Build de l'image Docker
2. Push vers ghcr.io
3. Déploiement (SSH ou Kubernetes)

### Secrets GitHub requis

| Secret | Description |
| ------ | ----------- |
| GHCR_TOKEN | Token GitHub Packages |
| STAGING_HOST | IP/hostname staging |
| STAGING_SSH_KEY | Clé SSH staging |
| PROD_HOST | IP/hostname production |
| PROD_SSH_KEY | Clé SSH production |
| DATABASE_URL | URL PostgreSQL |
| MONGODB_URI | URI MongoDB |

## Configuration Nginx (Reverse Proxy)

```nginx
upstream backend {
    server 127.0.0.1:3001;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name api.discord-clone.example.com;

    ssl_certificate /etc/letsencrypt/live/api.discord-clone.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.discord-clone.example.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # API routes
    location /api/ {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket
    location /ws {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 86400;
    }

    # Health check (internal)
    location /health {
        proxy_pass http://backend;
        allow 127.0.0.1;
        deny all;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name api.discord-clone.example.com;
    return 301 https://$server_name$request_uri;
}
```

## Migrations en production

### Avant déploiement

```bash
# Backup de la base
pg_dump -Fc discord_prod > backup_$(date +%Y%m%d).dump

# Appliquer les migrations
DATABASE_URL=$PROD_DATABASE_URL sqlx migrate run
```

### Rollback si nécessaire

```bash
# Annuler la dernière migration
DATABASE_URL=$PROD_DATABASE_URL sqlx migrate revert

# Ou restaurer le backup
pg_restore -d discord_prod backup_20260202.dump
```

## Monitoring

### Health check

```bash
curl https://api.discord-clone.example.com/health
```

Réponse attendue :

```json
{
  "status": "ok",
  "version": "1.0.0",
  "database": "connected",
  "mongodb": "connected"
}
```

### Logs

```bash
# Docker logs
docker logs -f backend

# Journalctl (si systemd)
journalctl -u discord-backend -f
```

### Métriques à surveiller

| Métrique | Seuil d'alerte |
| -------- | -------------- |
| CPU | > 80% pendant 5 min |
| Memory | > 85% |
| Disk | > 90% |
| Response time P99 | > 500ms |
| Error rate | > 1% |
| WebSocket connections | > 10000 |

## Scaling

### Horizontal scaling

```yaml
# docker-compose.prod.yml
services:
  backend:
    deploy:
      replicas: 3
```

Avec Nginx upstream :

```nginx
upstream backend {
    server backend1:3001;
    server backend2:3001;
    server backend3:3001;
    keepalive 32;
}
```

### Considérations WebSocket

Pour le scaling horizontal avec WebSocket :
1. Utiliser Redis Pub/Sub pour la synchronisation
2. Sticky sessions ou Redis pour l'état des sessions

## Rollback

### Procédure de rollback

```bash
# 1. Identifier la version précédente
docker images | grep discord-clone-backend

# 2. Rollback vers la version précédente
docker pull ghcr.io/epitechmscpropromo2028/t-jsf-600-par_20/backend:v1.0.0
docker-compose up -d

# 3. Rollback des migrations si nécessaire
sqlx migrate revert

# 4. Vérifier le health check
curl https://api.discord-clone.example.com/health
```

## Checklist de déploiement

### Avant déploiement

- [ ] Tests passent en CI
- [ ] Migrations testées sur staging
- [ ] Backup de la base de données
- [ ] Variables d'environnement configurées
- [ ] Certificats SSL valides
- [ ] Plan de rollback prêt

### Après déploiement

- [ ] Health check OK
- [ ] Logs sans erreurs
- [ ] Métriques normales
- [ ] Tests smoke passent
- [ ] Notifier l'équipe

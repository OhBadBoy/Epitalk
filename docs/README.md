# Documentation Discord Clone

Bienvenue dans la documentation technique du projet Discord Clone.

## Structure

```
docs/
├── README.md                 # Ce fichier
├── api/
│   ├── README.md            # Vue d'ensemble API REST
│   ├── openapi.yaml         # Spécification OpenAPI 3.1
│   ├── auth.md              # Documentation authentification
│   ├── servers.md           # Documentation servers
│   ├── channels.md          # Documentation channels
│   ├── members.md           # Documentation members
│   └── invites.md           # Documentation invites
├── websocket/
│   ├── README.md            # Vue d'ensemble WebSocket
│   └── protocol.md          # Protocole WebSocket détaillé
├── architecture/
│   ├── README.md            # Architecture globale
│   ├── database.md          # Schéma base de données
│   └── security.md          # Sécurité et authentification
└── guides/
    ├── getting-started.md   # Guide de démarrage
    └── deployment.md        # Guide de déploiement
```

## Liens Rapides

- [🔐 API Authentication](./api/auth.md)
- [📡 API REST](./api/README.md)
- [🔌 WebSocket Protocol](./websocket/protocol.md)
- [🏗️ Architecture](./architecture/README.md)
- [🚀 Getting Started](./guides/getting-started.md)

## Stack Technique

| Composant | Technologie |
|-----------|-------------|
| Backend | Rust + Axum |
| Base de données relationnelle | PostgreSQL 16 |
| Base de données messages | MongoDB 7 |
| Authentification | JWT (RS256) |
| Temps réel | WebSocket |
| CI/CD | GitHub Actions |
| Conteneurisation | Docker |

## Conventions

- **API REST** : Préfixe `/api/v1`
- **WebSocket** : Endpoint `/ws`
- **Format** : JSON (application/json)
- **Authentification** : Bearer Token JWT
- **Dates** : ISO 8601 (UTC)
- **IDs** : UUID v4

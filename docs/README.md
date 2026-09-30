# Documentation Discord Clone

Bienvenue dans la documentation technique du projet Discord Clone.

## Équipe, branches et UML

Le [référentiel des rôles, des 14 branches et des correspondances UML](REPARTITION_ROLES_BRANCHES_UML.md) définit le périmètre de James, Daouda, Moïse et Hadrian. Il contient les fonctionnalités et achievements associés ainsi que la répartition FRONTEND / BACKEND / DB.

- [Index des 14 branches et owners](../backend/BRANCHES.md)
- [Diagrammes UML et correspondance avec les branches](uml/README.md)

Les diagrammes et les audits RTC conservés dans ce dossier décrivent des étapes historiques. Le référentiel des responsabilités ne vaut pas validation de l'état d'implémentation ou des achievements.

## Structure

```
docs/
├── README.md                 # Ce fichier
├── REPARTITION_ROLES_BRANCHES_UML.md # Rôles, branches, fonctionnalités et achievements
├── uml/                      # Index UML, sources PlantUML et images historiques
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
    └── deployment.md        # Guide de déploiement
```

## Liens Rapides

- [🔐 API Authentication](./api/auth.md)
- [📡 API REST](./api/README.md)
- [🔌 WebSocket Protocol](./websocket/protocol.md)
- [🏗️ Architecture](./architecture/README.md)
- [🚀 Démarrage](../README.md#démarrage-en-local)

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

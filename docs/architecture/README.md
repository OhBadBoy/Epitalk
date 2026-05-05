# Architecture - Vue d'ensemble

## Diagramme C4 - Contexte

```
┌─────────────────────────────────────────────────────────────────┐
│                        Discord Clone                             │
│                                                                  │
│  ┌──────────┐      ┌──────────────────┐      ┌──────────────┐  │
│  │  Client  │◄────►│   Backend API    │◄────►│  PostgreSQL  │  │
│  │  (Web)   │      │   (Rust/Axum)    │      │              │  │
│  └──────────┘      └────────┬─────────┘      └──────────────┘  │
│                             │                                    │
│                             │                 ┌──────────────┐  │
│                             └────────────────►│   MongoDB    │  │
│                                               │  (Messages)  │  │
│                                               └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Stack Technique

### Backend

| Composant | Technologie | Version |
| --------- | ----------- | ------- |
| Runtime | Rust | 1.75+ |
| Framework HTTP | Axum | 0.7 |
| WebSocket | tokio-tungstenite | 0.21 |
| ORM | SQLx | 0.7 |
| Auth | jsonwebtoken | 9.x |
| Password Hash | argon2 | 0.5 |
| Validation | validator | 0.16 |
| Serialization | serde | 1.x |

### Bases de données

| Type | Technologie | Usage |
| ---- | ----------- | ----- |
| Relationnelle | PostgreSQL 16 | Users, Servers, Channels, Members |
| Document | MongoDB 7 | Messages, Attachments |

### Infrastructure

| Composant | Technologie |
| --------- | ----------- |
| Container | Docker |
| CI/CD | GitHub Actions |
| Registry | ghcr.io |

## Architecture Hexagonale

```
┌───────────────────────────────────────────────────────────────┐
│                        Adapters (Entrants)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │
│  │ HTTP Routes │  │  WebSocket  │  │   CLI / Scheduled   │   │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘   │
│         │                │                     │               │
├─────────┴────────────────┴─────────────────────┴───────────────┤
│                        Application Layer                        │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                      Use Cases                            │ │
│  │  • CreateServer  • SendMessage  • JoinServer             │ │
│  │  • CreateChannel • ManageRoles  • Authenticate           │ │
│  └──────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                         Domain Layer                            │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                       Entities                            │ │
│  │  • User  • Server  • Channel  • Message  • Member        │ │
│  └──────────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    Domain Services                        │ │
│  │  • RolePermissions  • InviteValidator                    │ │
│  └──────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                        Adapters (Sortants)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │
│  │ PostgreSQL  │  │   MongoDB   │  │   External APIs     │   │
│  │ Repository  │  │ Repository  │  │   (Email, CDN...)   │   │
│  └─────────────┘  └─────────────┘  └─────────────────────┘   │
└───────────────────────────────────────────────────────────────┘
```

## Structure des fichiers

```
backend/
├── Cargo.toml
├── Dockerfile
├── src/
│   ├── main.rs              # Point d'entrée
│   ├── config.rs            # Configuration
│   ├── state.rs             # État partagé
│   ├── error.rs             # Gestion d'erreurs
│   │
│   ├── auth/                # Module authentification
│   │   ├── mod.rs
│   │   ├── jwt.rs           # Génération/validation JWT
│   │   ├── password.rs      # Hashing Argon2
│   │   └── middleware.rs    # Middleware extraction user
│   │
│   ├── models/              # Entités du domaine
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── server.rs
│   │   ├── channel.rs
│   │   ├── membership.rs
│   │   └── invite.rs
│   │
│   ├── repositories/        # Accès aux données
│   │   ├── mod.rs
│   │   ├── user_repo.rs
│   │   ├── server_repo.rs
│   │   ├── channel_repo.rs
│   │   ├── membership_repo.rs
│   │   └── invite_repo.rs
│   │
│   ├── routes/              # Endpoints HTTP
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── servers.rs
│   │   ├── channels.rs
│   │   ├── members.rs
│   │   ├── invites.rs
│   │   └── health.rs
│   │
│   ├── ws/                  # WebSocket (à implémenter)
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   ├── hub.rs
│   │   └── messages.rs
│   │
│   └── db/                  # Connexions DB
│       ├── mod.rs
│       └── postgres.rs
│
└── migrations/              # Migrations SQL
    └── 001_initial.sql
```

## Flux de données

### Requête HTTP

```
Client → Middleware Auth → Route Handler → Repository → Database
                                    ↓
                              Response ← Serialization
```

### WebSocket Message

```
Client ──WebSocket──► Hub ──Broadcast──► Subscribers
            │
            └──► Repository ──► MongoDB (persist)
```

## Scalabilité

### Horizontal Scaling

```
                    ┌──────────────┐
                    │ Load Balancer│
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
    ┌─────┴─────┐    ┌─────┴─────┐    ┌─────┴─────┐
    │ Backend 1 │    │ Backend 2 │    │ Backend 3 │
    └─────┬─────┘    └─────┬─────┘    └─────┬─────┘
          │                │                │
          └────────────────┼────────────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
        ┌─────┴─────┐            ┌──────┴──────┐
        │ PostgreSQL│            │    Redis    │
        │  Primary  │            │   Pub/Sub   │
        └───────────┘            └─────────────┘
```

### État partagé

- **Sessions** : Redis
- **WebSocket Pub/Sub** : Redis
- **Cache** : Redis
- **Données persistantes** : PostgreSQL (primary + replicas)

## Liens

- [Schéma base de données](./database.md)
- [Sécurité](./security.md)

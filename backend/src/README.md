# Backend: Servers, Channels & Members

> Branch: `feat/backend-servers-channels-members/james`

## 📋 Objectif

Implémentation des routes REST et repositories pour la gestion des :
- **Servers** (serveurs/communautés)
- **Channels** (canaux de discussion)
- **Members** (membres et rôles)
- **Invites** (invitations)

## 🗂️ Structure

```
backend/src/
├── main.rs                 # Point d'entrée
├── config.rs               # Configuration
├── state.rs                # AppState partagé
├── error.rs                # Types d'erreur
├── db/
│   ├── mod.rs
│   └── postgres.rs         # Pool PostgreSQL
├── models/
│   ├── mod.rs
│   ├── user.rs             # User model
│   ├── server.rs           # Server model
│   ├── membership.rs       # Membership + MemberRole
│   ├── channel.rs          # Channel + ChannelKind
│   └── invite.rs           # Invite model
├── repositories/
│   ├── mod.rs
│   ├── user_repo.rs        # User CRUD
│   ├── server_repo.rs      # Server CRUD
│   ├── membership_repo.rs  # Membership CRUD
│   ├── channel_repo.rs     # Channel CRUD
│   └── invite_repo.rs      # Invite CRUD
└── routes/
    ├── mod.rs              # Router principal
    ├── health.rs           # GET /api/health
    ├── servers.rs          # /api/servers/*
    ├── channels.rs         # /api/servers/:id/channels/*
    ├── members.rs          # /api/servers/:id/members/*
    └── invites.rs          # /api/servers/:id/invites/*
```

## 🚀 API Endpoints

### Health Check

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |

### Servers

| Method | Endpoint | Auth | Role | Description |
|--------|----------|------|------|-------------|
| GET | `/api/servers` | ✅ | * | List user's servers |
| POST | `/api/servers` | ✅ | * | Create a server |
| GET | `/api/servers/:id` | ✅ | MEMBER+ | Get server details |
| PATCH | `/api/servers/:id` | ✅ | ADMIN+ | Update server |
| DELETE | `/api/servers/:id` | ✅ | OWNER | Delete server |
| POST | `/api/servers/:id/leave` | ✅ | MEMBER/ADMIN | Leave server |
| POST | `/api/servers/:id/transfer` | ✅ | OWNER | Transfer ownership |

### Channels

| Method | Endpoint | Auth | Role | Description |
|--------|----------|------|------|-------------|
| GET | `/api/servers/:id/channels` | ✅ | MEMBER+ | List channels |
| POST | `/api/servers/:id/channels` | ✅ | ADMIN+ | Create channel |
| GET | `/api/servers/:id/channels/:cid` | ✅ | MEMBER+ | Get channel |
| PATCH | `/api/servers/:id/channels/:cid` | ✅ | ADMIN+ | Update channel |
| DELETE | `/api/servers/:id/channels/:cid` | ✅ | ADMIN+ | Delete channel |

### Members

| Method | Endpoint | Auth | Role | Description |
|--------|----------|------|------|-------------|
| GET | `/api/servers/:id/members` | ✅ | MEMBER+ | List members |
| GET | `/api/servers/:id/members/:uid` | ✅ | MEMBER+ | Get member |
| PATCH | `/api/servers/:id/members/:uid/role` | ✅ | OWNER | Update role |
| DELETE | `/api/servers/:id/members/:uid` | ✅ | ADMIN+ | Kick member |

### Invites

| Method | Endpoint | Auth | Role | Description |
|--------|----------|------|------|-------------|
| GET | `/api/servers/:id/invites` | ✅ | ADMIN+ | List invites |
| POST | `/api/servers/:id/invites` | ✅ | ADMIN+ | Create invite |
| DELETE | `/api/servers/:id/invites/:iid` | ✅ | ADMIN+ | Delete invite |
| POST | `/api/join` | ✅ | * | Join via invite code |

## 🔐 RBAC (Role-Based Access Control)

```
OWNER > ADMIN > MEMBER
```

| Permission | MEMBER | ADMIN | OWNER |
|------------|:------:|:-----:|:-----:|
| View servers/channels/members | ✅ | ✅ | ✅ |
| Create servers | ✅ | ✅ | ✅ |
| Create channels | ❌ | ✅ | ✅ |
| Update channels | ❌ | ✅ | ✅ |
| Delete channels | ❌ | ✅ | ✅ |
| Create invites | ❌ | ✅ | ✅ |
| Kick members | ❌ | ✅* | ✅ |
| Manage roles | ❌ | ❌ | ✅ |
| Transfer ownership | ❌ | ❌ | ✅ |
| Delete server | ❌ | ❌ | ✅ |
| Leave server | ✅ | ✅ | ❌ |

*ADMIN can only kick MEMBER, not other ADMIN

## 📦 Dependencies

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono"] }
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
validator = { version = "0.18", features = ["derive"] }
tracing = "0.1"
```

## ⚙️ Configuration

Variables d'environnement (`.env`):

```bash
DATABASE_URL=postgres://epitalk:Epitalk94!@localhost:5432/epitalk
PORT=3001
RUST_LOG=epitalk_backend=debug,tower_http=debug
```

## 🧪 Exécution

```bash
cd backend

# Démarrer les BDD (depuis la branche feat/db-postgres-schema-migrations)
docker compose up -d

# Lancer le backend
cargo run

# Le serveur écoute sur http://localhost:3001
```

## 📝 Request/Response Examples

### Create Server

```bash
POST /api/servers
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Server"
}
```

```json
{
  "id": "uuid",
  "name": "My Server",
  "owner_id": "uuid",
  "created_at": "2026-02-02T...",
  "member_count": 1
}
```

### Create Channel

```bash
POST /api/servers/:server_id/channels
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "general",
  "kind": "TEXT"
}
```

### Create Invite

```bash
POST /api/servers/:server_id/invites
Authorization: Bearer <token>
Content-Type: application/json

{
  "expires_in_hours": 24,
  "max_uses": 10
}
```

```json
{
  "id": "uuid",
  "server_id": "uuid",
  "code": "abc12345",
  "expires_at": "2026-02-03T...",
  "max_uses": 10,
  "use_count": 0,
  "is_valid": true
}
```

### Join Server

```bash
POST /api/join
Authorization: Bearer <token>
Content-Type: application/json

{
  "code": "abc12345"
}
```

## ⚠️ TODO

- [ ] **Auth Middleware** (`feat/backend-auth-rbac/james`)
  - JWT validation
  - Extract user_id from token
  - Inject into request context

- [ ] **WebSocket Integration** (`feat/backend-ws-hub-messages/daouda`)
  - Notify on member join/leave
  - Notify on channel create/delete

## 🔗 Liens

- **UML Backend** : `docs/uml/epitalk_backend.puml`
- **Schema DB** : `feat/db-postgres-schema-migrations/james`
- **Auth** : `feat/backend-auth-rbac/james`

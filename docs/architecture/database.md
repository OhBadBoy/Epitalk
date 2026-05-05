# Schéma Base de Données

## Vue d'ensemble

Le projet utilise deux bases de données :

- **PostgreSQL** : Données relationnelles (users, servers, channels, members, invites)
- **MongoDB** : Messages et attachments (optimisé pour l'écriture/lecture séquentielle)

## PostgreSQL - Schéma

### Diagramme ERD

```
┌─────────────────┐       ┌─────────────────┐
│     users       │       │    servers      │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ email           │       │ name            │
│ username        │       │ description     │
│ password_hash   │       │ icon_url        │
│ avatar_url      │       │ owner_id (FK)   │──┐
│ created_at      │       │ created_at      │  │
│ updated_at      │       │ updated_at      │  │
└────────┬────────┘       └────────┬────────┘  │
         │                         │           │
         │    ┌────────────────────┘           │
         │    │                                │
         │    │  ┌─────────────────┐           │
         │    │  │   memberships   │           │
         │    │  ├─────────────────┤           │
         │    └─►│ server_id (FK)  │◄──────────┘
         └──────►│ user_id (FK)    │
                 │ role            │
                 │ joined_at       │
                 └─────────────────┘

┌─────────────────┐       ┌─────────────────┐
│    channels     │       │    invites      │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ server_id (FK)  │       │ code (UNIQUE)   │
│ name            │       │ server_id (FK)  │
│ description     │       │ created_by (FK) │
│ channel_type    │       │ uses            │
│ position        │       │ max_uses        │
│ created_at      │       │ expires_at      │
│ updated_at      │       │ created_at      │
└─────────────────┘       └─────────────────┘
```

### Table: users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(32) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### Table: servers

```sql
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    icon_url TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_servers_owner ON servers(owner_id);
```

### Table: channels

```sql
CREATE TYPE channel_type AS ENUM ('text', 'voice');

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    channel_type channel_type NOT NULL DEFAULT 'text',
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_id, name)
);

CREATE INDEX idx_channels_server ON channels(server_id);
```

### Table: memberships

```sql
CREATE TYPE member_role AS ENUM ('owner', 'admin', 'moderator', 'member');

CREATE TABLE memberships (
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role member_role NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, user_id)
);

CREATE INDEX idx_memberships_user ON memberships(user_id);
CREATE INDEX idx_memberships_server ON memberships(server_id);
```

### Table: invites

```sql
CREATE TABLE invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(8) NOT NULL UNIQUE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    uses INTEGER NOT NULL DEFAULT 0,
    max_uses INTEGER,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invites_code ON invites(code);
CREATE INDEX idx_invites_server ON invites(server_id);
```

## MongoDB - Schéma

### Collection: messages

```javascript
{
  "_id": ObjectId("..."),
  "channel_id": "770e8400-e29b-41d4-a716-446655440001",
  "author_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Hello, World!",
  "attachments": [
    {
      "url": "https://cdn.example.com/files/abc123.png",
      "filename": "image.png",
      "content_type": "image/png",
      "size": 12345
    }
  ],
  "mentions": ["550e8400-e29b-41d4-a716-446655440001"],
  "reply_to": null,
  "edited_at": null,
  "created_at": ISODate("2026-02-02T10:00:00Z")
}
```

### Index MongoDB

```javascript
// Index composé pour requêtes par channel + tri temporel
db.messages.createIndex({ "channel_id": 1, "created_at": -1 })

// Index pour recherche de messages par auteur
db.messages.createIndex({ "author_id": 1, "created_at": -1 })

// Index TTL pour auto-suppression (optionnel, ex: 90 jours)
// db.messages.createIndex({ "created_at": 1 }, { expireAfterSeconds: 7776000 })
```

## Migrations

Les migrations SQL sont gérées avec `sqlx-cli`.

### Commandes

```bash
# Créer une migration
sqlx migrate add <name>

# Appliquer les migrations
sqlx migrate run

# Annuler la dernière migration
sqlx migrate revert

# Vérifier le statut
sqlx migrate info
```

### Structure des migrations

```
backend/migrations/
├── 20260101000000_create_users.sql
├── 20260101000001_create_servers.sql
├── 20260101000002_create_channels.sql
├── 20260101000003_create_memberships.sql
└── 20260101000004_create_invites.sql
```

## Considérations de performance

### Index recommandés

| Table | Index | Raison |
| ----- | ----- | ------ |
| users | email | Login par email |
| users | username | Recherche utilisateur |
| servers | owner_id | Liste serveurs par propriétaire |
| channels | server_id | Liste channels d'un serveur |
| memberships | user_id | Liste serveurs d'un utilisateur |
| memberships | server_id | Liste membres d'un serveur |
| invites | code | Lookup invitation |

### Partitionnement (si nécessaire)

Pour les messages MongoDB avec volume important :

```javascript
// Shard par channel_id pour distribuer la charge
sh.shardCollection("discord.messages", { "channel_id": "hashed" })
```

## Backup et restauration

### PostgreSQL

```bash
# Backup
pg_dump -Fc discord > backup.dump

# Restore
pg_restore -d discord backup.dump
```

### MongoDB

```bash
# Backup
mongodump --db discord --out ./backup

# Restore
mongorestore --db discord ./backup/discord
```

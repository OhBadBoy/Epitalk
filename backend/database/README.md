# PostgreSQL Schema & Migrations

> Branch: `feat/db-postgres-schema-migrations/james`

## 📋 Objectif

Mise en place du schéma PostgreSQL pour l'application EpiTalk (Real Time Chat) :
- **5 tables** : users, servers, memberships, channels, invites
- **Contraintes** : FK, UNIQUE, CHECK, indexes
- **Migrations** : versionnées et transactionnelles
- **Docker** : environnement de développement prêt à l'emploi

## 🗂️ Structure

```
backend/
├── database/
│   ├── README.md                   # Ce fichier
│   ├── schema.sql                  # Schéma complet (référence)
│   └── migrations/
│       ├── 000_rollback_all.sql    # ⚠️ Rollback (DESTRUCTIF)
│       ├── 001_initial_schema.sql  # Tables + indexes + triggers
│       └── 002_seed_data.sql       # Données de test (dev)
├── docker-compose.yml              # PostgreSQL + MongoDB + Adminer
├── .env.example                    # Variables d'environnement
└── README.md                       # Documentation backend
```

## 🚀 Quick Start

### 1. Démarrer les bases de données

```bash
cd backend

# PostgreSQL + MongoDB
docker compose up -d

# Avec Adminer (interface web)
docker compose --profile dev up -d
```

### 2. Appliquer les migrations

```bash
# Migration initiale
psql -h localhost -U EpiTalk -d EpiTalk -f database/migrations/001_initial_schema.sql

# Données de test (dev uniquement)
psql -h localhost -U EpiTalk -d EpiTalk -f database/migrations/002_seed_data.sql
```

### 3. Vérifier

```bash
# Connexion psql
psql -h localhost -U EpiTalk -d EpiTalk
# Password: EpiTalk_password

# Ou via Adminer
open http://localhost:3001
```

## 📊 Schéma de Base de Données

### Diagramme ER

```
┌─────────────────┐       ┌─────────────────┐
│     users       │       │    servers      │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │◄──────┤ owner_id (FK)   │
│ email (UNIQUE)  │       │ id (PK)         │
│ password_hash   │       │ name            │
│ username        │       │ created_at      │
│ created_at      │       │ updated_at      │
│ updated_at      │       └────────┬────────┘
└────────┬────────┘                │
         │                         │
         │    ┌────────────────────┤
         │    │                    │
         ▼    ▼                    ▼
┌─────────────────┐       ┌─────────────────┐
│  memberships    │       │    channels     │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ user_id (FK)    │       │ server_id (FK)  │
│ server_id (FK)  │       │ name            │
│ role (ENUM)     │       │ kind (ENUM)     │
│ joined_at       │       │ created_at      │
│ (UNIQUE user+srv)│      │ updated_at      │
└─────────────────┘       └─────────────────┘
                                   
         ┌─────────────────┐
         │    invites      │
         ├─────────────────┤
         │ id (PK)         │
         │ server_id (FK)  │
         │ code (UNIQUE)   │
         │ created_by (FK) │
         │ expires_at      │
         │ max_uses        │
         │ use_count       │
         │ created_at      │
         └─────────────────┘
```

### Types ENUM

```sql
-- Rôles des membres
CREATE TYPE member_role AS ENUM ('OWNER', 'ADMIN', 'MEMBER');

-- Types de channels
CREATE TYPE channel_kind AS ENUM ('TEXT');
```

### Tables

| Table | Description | Colonnes clés |
|-------|-------------|---------------|
| `users` | Comptes utilisateurs | email (citext), password_hash, username |
| `servers` | Serveurs/communautés | name, owner_id → users |
| `memberships` | Relation User↔Server | user_id, server_id, role |
| `channels` | Canaux de discussion | server_id, name, kind |
| `invites` | Codes d'invitation | code, expires_at, max_uses |

### Indexes

| Index | Table | Colonnes | Type |
|-------|-------|----------|------|
| `idx_users_email` | users | email | UNIQUE (implicite) |
| `idx_memberships_user` | memberships | user_id | B-tree |
| `idx_memberships_server` | memberships | server_id | B-tree |
| `idx_channels_server` | channels | server_id | B-tree |
| `idx_invites_code` | invites | code | UNIQUE (implicite) |
| `idx_invites_server` | invites | server_id | B-tree |

## 🔐 Rôles & Permissions (RBAC)

| Permission | MEMBER | ADMIN | OWNER |
|------------|:------:|:-----:|:-----:|
| Écrire des messages | ✅ | ✅ | ✅ |
| Supprimer ses messages | ✅ | ✅ | ✅ |
| Voir les membres/online | ✅ | ✅ | ✅ |
| Créer des channels | ❌ | ✅ | ✅ |
| Modifier des channels | ❌ | ✅ | ✅ |
| Supprimer des channels | ❌ | ✅ | ✅ |
| Supprimer messages d'autres | ❌ | ✅ | ✅ |
| Créer des invitations | ❌ | ✅ | ✅ |
| Gérer les rôles | ❌ | ❌ | ✅ |
| Transférer ownership | ❌ | ❌ | ✅ |
| Quitter le serveur | ✅ | ✅ | ❌ |

## ⚙️ Configuration

### Variables d'environnement

```bash
# Copier le template
cp .env.example .env
```

| Variable | Description | Défaut |
|----------|-------------|--------|
| `DATABASE_URL` | PostgreSQL connection | `postgres://EpiTalk:EpiTalk_password@localhost:5432/EpiTalk` |
| `MONGO_URL` | MongoDB connection | `mongodb://EpiTalk:EpiTalk_password@localhost:27017/EpiTalk` |
| `JWT_SECRET` | Secret JWT | ⚠️ Changer en prod |
| `JWT_EXPIRES_IN` | Durée token | `7d` |

## 🧪 Données de Test

La migration `002_seed_data.sql` crée :

| Utilisateur | Email | Password (hash) |
|-------------|-------|-----------------|
| Alice | alice@example.com | `hashed_password_alice` |
| Bob | bob@example.com | `hashed_password_bob` |
| Charlie | charlie@example.com | `hashed_password_charlie` |
| Diana | diana@example.com | `hashed_password_diana` |

| Serveur | Owner | Membres |
|---------|-------|---------|
| Gaming Hub | Alice (OWNER) | Bob (ADMIN), Charlie (MEMBER) |
| Dev Team | Bob (OWNER) | Alice (MEMBER) |
| Music Lovers | Charlie (OWNER) | - |

## 🔄 Rollback

⚠️ **ATTENTION** : Supprime TOUTES les données !

```bash
psql -h localhost -U EpiTalk -d EpiTalk -f database/migrations/000_rollback_all.sql
```

## 📝 Conventions

### Migrations

- Préfixe numérique : `XXX_description.sql`
- Toujours transactionnelles (`BEGIN; ... COMMIT;`)
- Idempotentes quand possible (`IF NOT EXISTS`)
- Enregistrées dans `_migrations`

### Nommage SQL

- Tables : `snake_case` pluriel (`users`, `servers`)
- Colonnes : `snake_case` (`created_at`, `user_id`)
- FK : `table_id` (ex: `server_id` → `servers.id`)
- Indexes : `idx_table_column`
- Contraintes : `chk_table_rule`, `uq_table_columns`

## 🔗 Liens

- **UML** : `docs/uml/EpiTalk_DB.puml`
- **Cahier des charges** : `project.pdf`
- **Branche suivante** : `feat/backend-auth-rbac/james`

# EpiTalk Backend Database

## Structure

```
backend/
├── database/
│   ├── schema.sql              # Full schema (documentation)
│   └── migrations/
│       ├── 000_rollback_all.sql    # Rollback script (DESTRUCTIVE)
│       ├── 001_initial_schema.sql  # Initial tables
│       └── 002_seed_data.sql       # Test data (dev only)
├── docker-compose.yml          # PostgreSQL + MongoDB + Adminer
├── .env.example                # Environment variables template
└── README.md                   # This file
```

## Quick Start

### 1. Start databases with Docker

```bash
# Start PostgreSQL + MongoDB
docker compose up -d

# With Adminer GUI (development)
docker compose --profile dev up -d
```

### 2. Verify databases are running

```bash
docker compose ps
```

### 3. Connect to PostgreSQL

```bash
# Using psql
psql -h localhost -U epitalk -d epitalk

# Password: Epitalk94!
```

### 4. Run migrations manually (if needed)

```bash
# Apply initial schema
psql -h localhost -U epitalk -d epitalk -f database/migrations/001_initial_schema.sql

# Apply seed data (development only)
psql -h localhost -U epitalk -d epitalk -f database/migrations/002_seed_data.sql
```

### 5. Access Adminer GUI

Open http://localhost:3001 in your browser:
- **System**: PostgreSQL
- **Server**: postgres
- **Username: epitalk
- **Password**: Epitalk94!
- **Database: epitalk

## Database Schema

### Tables

| Table | Description |
|-------|-------------|
| `users` | User accounts (email, password_hash, username) |
| `servers` | Chat servers/communities |
| `memberships` | User-Server relationship with role (OWNER/ADMIN/MEMBER) |
| `channels` | Text channels within servers |
| `invites` | Invitation codes to join servers |
| `_migrations` | Migration tracking table |

### Entity Relationship

```
users (1) ──────── (N) servers (owner)
  │                      │
  │                      │
  └──── (N) memberships (N) ────┘
              │
              │ role: OWNER | ADMIN | MEMBER
              │
servers (1) ──┴── (N) channels
  │
  └── (N) invites
```

### Roles & Permissions

| Permission | MEMBER | ADMIN | OWNER |
|------------|--------|-------|-------|
| Write messages | ✅ | ✅ | ✅ |
| Delete own messages | ✅ | ✅ | ✅ |
| See members/online | ✅ | ✅ | ✅ |
| See typing indicator | ✅ | ✅ | ✅ |
| Create channels | ❌ | ✅ | ✅ |
| Update channels | ❌ | ✅ | ✅ |
| Delete channels | ❌ | ✅ | ✅ |
| Delete others' messages | ❌ | ✅ | ✅ |
| Create invites | ❌ | ✅ | ✅ |
| Manage roles | ❌ | ❌ | ✅ |
| Transfer ownership | ❌ | ❌ | ✅ |
| Leave server | ✅ | ✅ | ❌ |

## Environment Variables

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

Key variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://epitalk:Epitalk94!@localhost:5432/epitalk` |
| `MONGO_URL` | MongoDB connection string | `mongodb://epitalk:Epitalk94!@localhost:27017/epitalk` |
| `JWT_SECRET` | Secret for JWT signing | (change in production!) |

## Rollback

⚠️ **WARNING**: This will delete ALL data!

```bash
psql -h localhost -U epitalk -d epitalk -f database/migrations/000_rollback_all.sql
```

## MongoDB (Messages)

Messages are stored in MongoDB for better performance with large volumes.

Collection: `messages`

```javascript
{
  _id: ObjectId,
  channel_id: "uuid-string",
  server_id: "uuid-string", 
  author_id: "uuid-string",
  content: "Message text",
  created_at: ISODate,
  deleted_at: ISODate | null
}
```

Indexes (to be created in `feat/db-mongo-indexes/hadrian`):
- `{ channel_id: 1, created_at: -1 }` (primary)
- `{ server_id: 1, created_at: -1 }` (optional)
- `{ author_id: 1, created_at: -1 }` (optional)

## Coverage

You can run coverage with `cargo-llvm-cov` using Make targets:

```bash
# Full backend coverage summary
make coverage

# Core backend coverage summary (quality gate > 70%)
make coverage-core
```

Notes:
- Both targets set `EPITALK_SKIP_SLOW_TESTS=1` so long-running flow tests do not block coverage runs.
- `coverage-core` excludes bootstrap and out-of-scope files used for manual tools or non-core paths.

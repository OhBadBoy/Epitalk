# 🚀 EpiTalk - Real-Time Chat

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-000000?style=for-the-badge&logo=rust&logoColor=white)
![Next.js](https://img.shields.io/badge/Next.js_16-000000?style=for-the-badge&logo=next.js&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL_16-336791?style=for-the-badge&logo=postgresql&logoColor=white)
![MongoDB](https://img.shields.io/badge/MongoDB_7-47A248?style=for-the-badge&logo=mongodb&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white)

**Application de chat temps réel inspirée de Discord**

[Documentation](#-documentation) • [Quick Start](#-quick-start) • [Architecture](#-architecture) • [API](#-api-rest) • [Équipe](#-équipe)

</div>

---

## 📋 Table des matières

- [Fonctionnalités](#-fonctionnalités)
- [Stack Technique](#-stack-technique)
- [Quick Start](#-quick-start)
- [Architecture](#-architecture)
- [API REST](#-api-rest)
- [WebSocket](#-websocket)
- [Base de Données](#-base-de-données)
- [Documentation](#-documentation)
- [Tests](#-tests)
- [Équipe](#-équipe)

---

## ✨ Fonctionnalités

### 🔐 Authentification
- [x] Inscription / Connexion (JWT RS256)
- [x] Refresh token
- [x] Profil utilisateur (`/me`)
- [x] Validation des entrées

### 🏠 Serveurs
- [x] Création de serveurs
- [x] Liste des serveurs de l'utilisateur
- [x] Modification / Suppression (Owner only)
- [x] Système d'invitations avec expiration

### 📢 Channels
- [x] CRUD channels par serveur
- [x] Channels textuels
- [x] Permissions RBAC

### 👥 Membres & RBAC
- [x] Rôles : **Owner** > **Admin** > **Moderator** > **Member**
- [x] Gestion des membres (kick, promote, demote)
- [x] Vérification des permissions par middleware

### 💬 Messages Temps Réel
- [x] WebSocket bidirectionnel
- [x] Envoi/Réception instantanée
- [x] Historique des messages (MongoDB)
- [x] Rooms par channel
- [ ] Typing indicators
- [ ] Présence (online/offline/idle)

### 🎨 Interface
- [x] UI Discord-like (Shadcn + Tailwind)
- [x] Dark/Light mode ready
- [x] Responsive design
- [x] Sidebar serveurs/channels

---

## 🛠 Stack Technique

| Composant | Technologie | Version |
|-----------|-------------|---------|
| **Backend** | Rust + Axum | 1.75+ |
| **Frontend** | Next.js (App Router) | 16.x |
| **DB Relationnelle** | PostgreSQL | 16 |
| **DB Messages** | MongoDB | 7 |
| **Auth** | JWT RS256 | - |
| **Temps Réel** | WebSocket | - |
| **UI** | Shadcn/ui + Tailwind | - |
| **State** | Zustand | 5.x |
| **Container** | Docker Compose | - |
| **CI/CD** | GitHub Actions | - |

---

## 🚀 Quick Start

### Prérequis

- [Docker](https://www.docker.com/) & Docker Compose
- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) ou npm

### 1. Cloner le projet

```bash
git clone https://github.com/EpitechMscProPromo2028/T-JSF-600-PAR_20.git
cd T-JSF-600-PAR_20
git checkout Test
```

### 2. Lancer les bases de données

```bash
cd backend
docker-compose up -d
```

Vérifier que les containers tournent :
```bash
docker-compose ps
# epitalk-postgres   ✅ Running
# epitalk-mongodb    ✅ Running
```

### 3. Configurer l'environnement backend

```bash
cp .env.example .env
# Modifier les variables si nécessaire
```

**Variables d'environnement :**
```env
DATABASE_URL=postgres://epitalk:Epitalk94!@localhost:5432/epitalk
MONGO_URL=mongodb://localhost:27017/epitalk
JWT_SECRET=your-super-secret-key-at-least-32-chars
JWT_EXPIRATION_HOURS=24
PORT=3001
```

### 4. Lancer le backend

```bash
cd backend
cargo run
```

```
🚀 Server listening on 0.0.0.0:3001
```

### 5. Lancer le frontend

```bash
cd frontend/real-time-chat
npm install
npm run dev
```

```
▲ Next.js 16.1.6
- Local: http://localhost:3000
```

### 6. Accéder à l'application

| Service | URL |
|---------|-----|
| **Frontend** | http://localhost:3000 |
| **API REST** | http://localhost:3001/api |
| **WebSocket** | ws://localhost:3001/ws |
| **Health Check** | http://localhost:3001/health |

---

## 🏗 Architecture

```
T-JSF-600-PAR_20/
├── backend/                    # API Rust/Axum
│   ├── src/
│   │   ├── auth/              # JWT, Password, Middleware
│   │   ├── models/            # Entités (User, Server, Channel...)
│   │   ├── repositories/      # Accès DB (PostgreSQL)
│   │   ├── routes/            # Endpoints REST
│   │   ├── services/          # Logique métier
│   │   ├── ws/                # WebSocket (Hub, Protocol)
│   │   └── main.rs
│   ├── database/
│   │   ├── schema.sql         # Schéma complet
│   │   └── migrations/        # Migrations SQL
│   └── docker-compose.yml
│
├── frontend/real-time-chat/    # UI Next.js
│   ├── app/
│   │   ├── (auth)/            # Pages login/register
│   │   ├── (app)/             # Pages authentifiées
│   │   └── api/               # Route handlers
│   ├── components/            # Composants React
│   ├── store/                 # Zustand stores
│   └── lib/                   # Utils, API client
│
├── docs/                       # Documentation
│   ├── api/                   # Specs API REST
│   ├── uml/                   # Diagrammes PlantUML
│   ├── websocket/             # Protocole WS
│   └── architecture/          # Docs architecture
│
└── .github/
    └── workflows/             # CI/CD pipelines
```

### Diagrammes UML

Les diagrammes sont disponibles dans `docs/uml/` :

| Diagramme | Fichier |
|-----------|---------|
| Backend Components | `epitalk_backend.puml` |
| Frontend Components | `EPITALK_FRONTEND.puml` |
| Database ER | `EPITALK_DB.puml` |

---

## 📡 API REST

Base URL : `http://localhost:3001/api`

### Authentification

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/auth/register` | Inscription |
| `POST` | `/auth/login` | Connexion |
| `GET` | `/auth/me` | Profil courant |
| `POST` | `/auth/refresh` | Refresh token |

### Serveurs

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/servers` | Liste mes serveurs |
| `POST` | `/servers` | Créer un serveur |
| `GET` | `/servers/:id` | Détails serveur |
| `PATCH` | `/servers/:id` | Modifier serveur |
| `DELETE` | `/servers/:id` | Supprimer serveur |

### Channels

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/servers/:id/channels` | Liste channels |
| `POST` | `/servers/:id/channels` | Créer channel |
| `PATCH` | `/channels/:id` | Modifier channel |
| `DELETE` | `/channels/:id` | Supprimer channel |

### Membres

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/servers/:id/members` | Liste membres |
| `PATCH` | `/servers/:id/members/:user_id` | Modifier rôle |
| `DELETE` | `/servers/:id/members/:user_id` | Kick membre |

### Invitations

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/servers/:id/invites` | Liste invitations |
| `GET` | `/servers/:id/invites/active` | Invitations actives |
| `POST` | `/servers/:id/invites` | Créer invitation |
| `POST` | `/invites/:code/join` | Rejoindre serveur |
| `DELETE` | `/invites/:code` | Supprimer invitation |

> 📖 Documentation complète : [`docs/api/`](./docs/api/)

---

## 🔌 WebSocket

Endpoint : `ws://localhost:3001/ws`

### Messages Client → Serveur

```typescript
// Rejoindre un channel
{ "type": "JoinChannel", "channel_id": "uuid" }

// Quitter un channel
{ "type": "LeaveChannel", "channel_id": "uuid" }

// Envoyer un message
{ "type": "MessageSend", "channel_id": "uuid", "content": "Hello!" }
```

### Messages Serveur → Client

```typescript
// Nouveau message
{
  "type": "MessageNew",
  "id": "mongo-id",
  "channel_id": "uuid",
  "author_id": "uuid",
  "content": "Hello!",
  "created_at": "2026-02-02T14:00:00Z"
}

// Utilisateur rejoint
{ "type": "UserJoined", "user_id": "uuid", "channel_id": "uuid" }

// Utilisateur parti
{ "type": "UserLeft", "user_id": "uuid", "channel_id": "uuid" }
```

> 📖 Protocole complet : [`docs/websocket/protocol.md`](./docs/websocket/protocol.md)

---

## 🗄 Base de Données

### PostgreSQL (Organisation & RBAC)

```sql
-- Tables principales
users          -- Utilisateurs
servers        -- Serveurs
memberships    -- Appartenance (user <-> server) + rôle
channels       -- Channels par serveur
invites        -- Liens d'invitation

-- Rôles RBAC
ENUM member_role: 'OWNER', 'ADMIN', 'MODERATOR', 'MEMBER'
```

### MongoDB (Messages)

```javascript
// Collection: messages
{
  _id: ObjectId,
  channel_id: "uuid",
  server_id: "uuid",
  author_id: "uuid",
  content: "string",
  created_at: ISODate,
  deleted_at: ISODate | null
}

// Index recommandé
{ channel_id: 1, created_at: -1 }
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [`docs/api/README.md`](./docs/api/README.md) | Vue d'ensemble API |
| [`docs/api/openapi.yaml`](./docs/api/openapi.yaml) | Spec OpenAPI 3.1 |
| [`docs/websocket/protocol.md`](./docs/websocket/protocol.md) | Protocole WebSocket |
| [`docs/architecture/README.md`](./docs/architecture/README.md) | Architecture globale |
| [`docs/architecture/database.md`](./docs/architecture/database.md) | Schéma DB |
| [`docs/architecture/security.md`](./docs/architecture/security.md) | Sécurité |

---

## 🧪 Tests

### Backend

```bash
cd backend

# Tests unitaires
cargo test

# Tests avec logs
cargo test -- --nocapture

# Clippy (lint)
cargo clippy --all-targets
```

### Frontend

```bash
cd frontend/real-time-chat

# Lint
npm run lint

# Type check
npm run type-check
```

### CI/CD

Les pipelines GitHub Actions exécutent automatiquement :
- ✅ Formatage (rustfmt, prettier)
- ✅ Lint (clippy, eslint)
- ✅ Build
- ✅ Tests
- ✅ Sécurité (dependabot)

---

## 📊 Progression du Projet

| Module | Progression |
|--------|:-----------:|
| Documentation & UML | 80% |
| Backend REST API | 85% |
| Backend WebSocket | 55% |
| Backend Auth/RBAC | 100% |
| Frontend Pages | 50% |
| Frontend Components | 55% |
| Frontend API Client | 35% |
| CI/CD | 70% |
| **Total** | **~62%** |

---

## 👥 Équipe

| Membre | Responsabilités |
|--------|-----------------|
| **James** | Backend Auth, RBAC, Invites, Architecture |
| **Daouda** | WebSocket, Messages, MongoDB |
| **[Autres]** | Frontend, Tests, DevOps |

---

## 📝 Conventions

### Git

```bash
# Branches
main              # Production
develop           # Développement
Test              # Intégration tests
feat/xxx/name     # Features par développeur

# Commits
feat: add user registration
fix: resolve JWT expiration bug
docs: update API documentation
refactor: simplify auth middleware
test: add server creation tests
```

### Code

- **Backend** : Rust standard (rustfmt)
- **Frontend** : ESLint + Prettier
- **API** : RESTful, JSON, snake_case
- **Dates** : ISO 8601 (UTC)
- **IDs** : UUID v4

---

## 📄 License

Ce projet est développé dans le cadre du cursus **Epitech MSc Pro 2028**.

---

<div align="center">

**[⬆ Retour en haut](#-epitalk---real-time-chat)**

Made with ❤️ by the EpiTalk Team

</div>

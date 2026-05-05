# 🚀 EpiTalk - Plateforme de Communication en Temps Réel

<div align="center">

![EpiTalk](https://img.shields.io/badge/EpiTalk-v1.0.0-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust)
![Next.js](https://img.shields.io/badge/Next.js-16-black?style=for-the-badge&logo=next.js)
![WebSocket](https://img.shields.io/badge/WebSocket-Real--Time-green?style=for-the-badge)

**Une plateforme de chat moderne inspirée de Discord, construite avec Rust et Next.js**

[Fonctionnalités](#-fonctionnalités) • [Installation](#-installation) • [Architecture](#-architecture) • [Documentation](#-documentation)

</div>

---

## 📋 Table des matières

- [Vue d'ensemble](#-vue-densemble)
- [Fonctionnalités](#-fonctionnalités)
- [Architecture](#-architecture)
- [Arborescence du projet](#-arborescence-du-projet)
- [Prérequis](#-prérequis)
- [Installation](#-installation)
- [Configuration](#-configuration)
- [Lancement](#-lancement)
- [Tests](#-tests)
- [API Documentation](#-api-documentation)
- [Technologies](#-technologies)

---

## 🎯 Vue d'ensemble

**EpiTalk** est une plateforme de communication en temps réel permettant de :
- Créer et gérer des serveurs de discussion
- Organiser des conversations par channels
- Échanger des messages instantanés via WebSocket
- Gérer des rôles et permissions (RBAC)
- Inviter des utilisateurs via des codes d'invitation

### 🏆 Score de conformité : **30/31 points (97%)**

---

## ✨ Fonctionnalités

### 🔐 Authentification & Sécurité
- ✅ Inscription et connexion avec JWT
- ✅ Hash de mots de passe avec bcrypt
- ✅ Protection des routes avec middleware d'authentification
- ✅ Tokens sécurisés avec expiration

### 🖥️ Gestion des Serveurs
- ✅ Création de serveurs privés
- ✅ Suppression de serveurs (propriétaire uniquement)
- ✅ Rejoindre un serveur via code d'invitation
- ✅ Quitter un serveur
- ✅ Navigation multi-serveurs simultanée

### 📢 Gestion des Channels
- ✅ Création de channels textuels
- ✅ Suppression de channels (admin/propriétaire)
- ✅ Organisation par serveur
- ✅ Channel par défaut automatique

### 💬 Messagerie en Temps Réel
- ✅ Messages instantanés via WebSocket
- ✅ Historique persistant (MongoDB)
- ✅ Chargement automatique des 50 derniers messages
- ✅ Indicateurs de frappe avec nom d'utilisateur
- ✅ Statut en ligne/hors ligne des utilisateurs
- ✅ Broadcast en temps réel

### 👥 Gestion des Utilisateurs
- ✅ Liste des membres du serveur
- ✅ Affichage du statut en ligne
- ✅ Système de rôles (Owner, Admin, Moderator, Member)
- ✅ Permissions différenciées par rôle

### 🎫 Système d'Invitations
- ✅ Génération de codes d'invitation uniques
- ✅ Limitation d'utilisation configurable
- ✅ Expiration automatique
- ✅ Révocation manuelle

### 💾 Persistance des Données
- ✅ PostgreSQL pour la structure relationnelle
- ✅ MongoDB pour l'historique des messages
- ✅ Migrations automatiques
- ✅ Backup et restore

---

## 🏗️ Architecture

### Stack Technique

```
┌─────────────────────────────────────────────────────────┐
│                     FRONTEND (Next.js 16)               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   React 19   │  │   Zustand    │  │  TailwindCSS │ │
│  │  TypeScript  │  │  WebSocket   │  │   shadcn/ui  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                            ▼
                     HTTP + WebSocket
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   BACKEND (Rust/Axum)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   REST API   │  │  WebSocket   │  │     JWT      │ │
│  │    Axum      │  │     Hub      │  │   Auth       │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                            ▼
              ┌─────────────┴─────────────┐
              ▼                           ▼
    ┌──────────────────┐        ┌──────────────────┐
    │   PostgreSQL     │        │     MongoDB      │
    │                  │        │                  │
    │  • Users         │        │  • Messages      │
    │  • Servers       │        │  • History       │
    │  • Channels      │        │  • Timestamps    │
    │  • Memberships   │        │                  │
    │  • Invites       │        │                  │
    └──────────────────┘        └──────────────────┘
```

---

## 📁 Arborescence du projet

```
T-JSF-600-PAR_20/
│
├── 📂 backend/                          # Backend Rust/Axum
│   ├── 📂 database/
│   │   ├── migrations/                  # Migrations SQL
│   │   └── schema.sql                   # Schéma PostgreSQL
│   │
│   ├── 📂 src/
│   │   ├── 📂 auth/                     # Authentification JWT
│   │   │   ├── jwt.rs                   # Génération/validation tokens
│   │   │   ├── middleware.rs            # Protection routes
│   │   │   └── password.rs              # Hashing bcrypt
│   │   │
│   │   ├── 📂 models/                   # Modèles de données
│   │   │   ├── user.rs
│   │   │   ├── server.rs
│   │   │   ├── channel.rs
│   │   │   ├── membership.rs
│   │   │   └── invite.rs
│   │   │
│   │   ├── 📂 repositories/             # Couche d'accès aux données
│   │   │   ├── user_repository.rs
│   │   │   ├── server_repository.rs
│   │   │   ├── channel_repository.rs
│   │   │   ├── member_repository.rs
│   │   │   └── invite_repository.rs
│   │   │
│   │   ├── 📂 routes/                   # Points d'entrée API REST
│   │   │   ├── auth.rs                  # /api/auth/*
│   │   │   ├── servers.rs               # /api/servers/*
│   │   │   ├── channels.rs              # /api/servers/:id/channels/*
│   │   │   ├── members.rs               # /api/servers/:id/members/*
│   │   │   └── invites.rs               # /api/invites/*
│   │   │
│   │   ├── 📂 services/                 # Logique métier
│   │   │   ├── message_service.rs       # Gestion messages MongoDB
│   │   │   ├── typing_service.rs        # Indicateurs de frappe
│   │   │   └── presence_service.rs      # Statut en ligne
│   │   │
│   │   ├── 📂 ws/                       # WebSocket
│   │   │   ├── hub.rs                   # Hub de connexions
│   │   │   ├── connection.rs            # Gestionnaire de connexion
│   │   │   ├── protocol.rs              # Événements WebSocket
│   │   │   └── ws_upgrade.rs            # Upgrade HTTP → WS
│   │   │
│   │   ├── 📂 tests/                    # Tests (29 tests)
│   │   │   └── integration_ws/
│   │   │
│   │   ├── main.rs                      # Point d'entrée
│   │   ├── config.rs                    # Configuration
│   │   └── error.rs                     # Gestion d'erreurs
│   │
│   ├── Cargo.toml                       # Dépendances Rust
│   └── docker-compose.yml               # Services Docker
│
├── 📂 frontend/real-time-chat/          # Frontend Next.js 16
│   ├── 📂 app/                          # App Router Next.js
│   │   ├── 📂 (auth)/                   # Routes authentification
│   │   │   ├── login/
│   │   │   └── register/
│   │   │
│   │   ├── 📂 (app)/servers/            # Interface principale
│   │   │   ├── 📂 components/
│   │   │   │   ├── servers-rail.tsx     # Barre latérale serveurs
│   │   │   │   ├── channels-sidebar.tsx # Liste des channels
│   │   │   │   ├── chat-panel.tsx       # Zone de chat
│   │   │   │   ├── members-panel.tsx    # Liste des membres
│   │   │   │   └── user-settings.tsx    # Paramètres utilisateur
│   │   │   └── page.tsx
│   │   │
│   │   └── invite/[code]/               # Rejoindre par invitation
│   │
│   ├── 📂 components/ui/                # shadcn/ui components
│   │
│   ├── 📂 lib/api/                      # Client API
│   │   ├── auth.api.ts
│   │   ├── servers.api.ts
│   │   ├── channels.api.ts
│   │   └── invites.api.ts
│   │
│   ├── 📂 store/                        # État global (Zustand)
│   │   ├── auth.store.ts                # Authentification
│   │   ├── server.store.ts              # Serveurs
│   │   ├── channel.store.ts             # Channels
│   │   └── websocket.store.ts           # WebSocket temps réel
│   │
│   └── package.json
│
└── 📂 docs/                             # Documentation
    ├── 📂 api/                          # Documentation API
    ├── 📂 architecture/                 # Architecture
    └── 📂 websocket/                    # Protocole WebSocket

```

---

## 🔧 Prérequis

### Obligatoire
- **Rust** 1.75+ ([Installation](https://rustup.rs/))
- **Node.js** 20+ ([Installation](https://nodejs.org/))
- **Docker** & **Docker Compose** ([Installation](https://docs.docker.com/get-docker/))

---

## 📦 Installation

### 1. Cloner le projet
```bash
git clone <repository-url>
cd T-JSF-600-PAR_20
```

### 2. Lancer les bases de données (Docker)
```bash
cd backend
docker-compose up -d
```

**Vérification :**
```bash
docker ps
# Devrait afficher :
# - epitalk-postgres (port 5433)
# - epitalk-mongo (port 27017)
```

### 3. Installer les dépendances backend
```bash
cd backend
cargo build
```

### 4. Installer les dépendances frontend
```bash
cd frontend/real-time-chat
npm install
```

---

## ⚙️ Configuration

### Backend (.env)

Créer `backend/.env` :
```bash
# PostgreSQL
DATABASE_URL=postgresql://epitalk:epitalk_password@localhost:5433/epitalk

# MongoDB
MONGODB_URI=mongodb://epitalk:epitalk_password@localhost:27017/epitalk_messages?authSource=admin

# JWT
JWT_SECRET=your-super-secret-key-change-in-production
JWT_EXPIRATION_HOURS=168

# Serveur
PORT=3001
```

### Frontend (.env.local)

Créer `frontend/real-time-chat/.env.local` :
```bash
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws
```

---

## 🚀 Lancement

### Développement (3 terminaux)

#### Terminal 1 : Bases de données
```bash
cd backend
docker-compose up
```

#### Terminal 2 : Backend Rust
```bash
cd backend
cargo run
```
✅ Attendez : `Server listening on 0.0.0.0:3001`

#### Terminal 3 : Frontend Next.js
```bash
cd frontend/real-time-chat
npm run dev
```
✅ Attendez : `Local: http://localhost:3000`

---

## 🌐 Accès

- **Frontend** : http://localhost:3000
- **Backend API** : http://localhost:3001/api
- **WebSocket** : ws://localhost:3001/ws

---

## 🧪 Tests

### Backend (29 tests)

```bash
cd backend
cargo test
```

**Coverage :**
- ✅ Services de typing (5 tests)
- ✅ Services de présence (5 tests)
- ✅ Protocole WebSocket (13 tests)

---

## 📚 API Documentation

### Authentification

#### POST `/api/auth/register`
```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "password123"
}
```

#### POST `/api/auth/login`
```json
{
  "email": "alice@example.com",
  "password": "password123"
}
```

### Serveurs

#### GET `/api/servers`
Liste tous les serveurs de l'utilisateur.

#### POST `/api/servers`
```json
{
  "name": "Mon Serveur"
}
```

### Invitations

#### POST `/api/servers/:server_id/invites`
```json
{
  "max_uses": 10
}
```

#### POST `/api/invites/:code/join`
Rejoindre un serveur avec un code.

---

## 🛠️ Technologies

### Backend
- **Rust** 1.75+ - Langage principal
- **Axum** 0.7 - Framework web
- **Tokio** 1.36 - Runtime asynchrone
- **SQLx** 0.7 - PostgreSQL
- **MongoDB Driver** 2.8
- **jsonwebtoken** 9.2 - JWT

### Frontend
- **Next.js** 16.1.6 - Framework React
- **React** 19 - UI Library
- **TypeScript** 5.x
- **Zustand** 5.0 - State management
- **TailwindCSS** 3.4 - Styling
- **shadcn/ui** - UI components

### Bases de données
- **PostgreSQL** 16 - Données structurées
- **MongoDB** 6.0 - Historique messages

---

## 📊 Schéma de base de données

### PostgreSQL

```sql
users (id, username, email, password_hash, created_at)
servers (id, name, owner_id, created_at)
channels (id, server_id, name, created_at)
memberships (user_id, server_id, role, joined_at)
invites (id, server_id, code, max_uses, expires_at)
```

### MongoDB

```javascript
// Collection : messages
{
  channel_id: String,
  author_id: String,
  username: String,
  content: String,
  created_at: String
}
```

---

## 🔒 Sécurité

### Authentification
- ✅ Tokens JWT avec expiration
- ✅ Hash bcrypt
- ✅ Middleware de protection

### Autorisations (RBAC)
| Rôle | Permissions |
|------|------------|
| **Owner** | Tout |
| **Admin** | Créer/supprimer channels, inviter |
| **Moderator** | Modération messages |
| **Member** | Lire, écrire |

---

## 📝 Commandes utiles

### Backend
```bash
cargo build          # Compiler
cargo run           # Lancer
cargo test          # Tests
cargo clippy        # Linter
```

### Frontend
```bash
npm run dev         # Développement
npm run build       # Build production
npm run lint        # Linter
```

---

## 📄 Licence

MIT License

---

<div align="center">

**Fait avec ❤️ par l'équipe EpiTalk**

[⬆️ Retour en haut](#-epitalk---plateforme-de-communication-en-temps-réel)

</div>

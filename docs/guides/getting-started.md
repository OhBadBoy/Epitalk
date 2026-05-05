# Guide de démarrage

## Prérequis

### Logiciels requis

| Logiciel | Version | Installation |
| -------- | ------- | ------------ |
| Rust | 1.75+ | [rustup.rs](https://rustup.rs) |
| PostgreSQL | 16+ | [postgresql.org](https://www.postgresql.org/download/) |
| MongoDB | 7+ | [mongodb.com](https://www.mongodb.com/try/download/community) |
| Docker | 24+ | [docker.com](https://www.docker.com/products/docker-desktop/) |

### Installation Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
winget install Rustlang.Rustup
```

Vérifier l'installation :

```bash
rustc --version
cargo --version
```

## Configuration

### 1. Cloner le repository

```bash
git clone https://github.com/EpitechMscProPromo2028/T-JSF-600-PAR_20.git
cd T-JSF-600-PAR_20
```

### 2. Configurer les variables d'environnement

```bash
cd backend
cp .env.example .env
```

Éditer `.env` :

```bash
# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/discord_clone
MONGODB_URI=mongodb://localhost:27017/discord_clone

# JWT Keys
JWT_PRIVATE_KEY_PATH=./keys/jwt_private.pem
JWT_PUBLIC_KEY_PATH=./keys/jwt_public.pem

# Server
HOST=127.0.0.1
PORT=3001
RUST_LOG=info
```

### 3. Générer les clés JWT

```bash
mkdir -p keys

# Générer clé privée RSA 4096 bits
openssl genrsa -out keys/jwt_private.pem 4096

# Extraire clé publique
openssl rsa -in keys/jwt_private.pem -pubout -out keys/jwt_public.pem
```

### 4. Configurer PostgreSQL

```bash
# Créer la base de données
createdb discord_clone

# Ou via psql
psql -U postgres -c "CREATE DATABASE discord_clone;"
```

### 5. Installer sqlx-cli et exécuter les migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres

# Exécuter les migrations
sqlx migrate run
```

### 6. Configurer MongoDB

MongoDB devrait fonctionner sans configuration supplémentaire. Vérifier :

```bash
mongosh --eval "db.runCommand({ ping: 1 })"
```

## Lancement

### Mode développement

```bash
cd backend
cargo run
```

Le serveur démarre sur `http://localhost:3001`

### Vérifier que ça fonctionne

```bash
curl http://localhost:3001/health
# {"status":"ok","version":"1.0.0"}
```

## Avec Docker

### Build et run

```bash
cd backend

# Build l'image
docker build -t discord-clone-backend .

# Lancer avec docker-compose (recommandé)
docker-compose up -d
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  backend:
    build: ./backend
    ports:
      - "3001:3001"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/discord_clone
      - MONGODB_URI=mongodb://mongo:27017/discord_clone
    depends_on:
      - postgres
      - mongo

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: discord_clone
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  mongo:
    image: mongo:7
    volumes:
      - mongo_data:/data/db
    ports:
      - "27017:27017"

volumes:
  postgres_data:
  mongo_data:
```

## Tests

### Lancer les tests

```bash
cd backend

# Tous les tests
cargo test

# Tests avec output
cargo test -- --nocapture

# Tests spécifiques
cargo test auth::
```

### Tests d'intégration

Les tests d'intégration nécessitent une base de données de test :

```bash
# Créer la DB de test
createdb discord_clone_test

# Exécuter les migrations
DATABASE_URL=postgres://postgres:password@localhost/discord_clone_test sqlx migrate run

# Lancer les tests d'intégration
cargo test --test '*'
```

## Commandes utiles

### Cargo

```bash
# Build optimisé
cargo build --release

# Vérifier le code (sans compiler)
cargo check

# Formater le code
cargo fmt

# Linter
cargo clippy

# Audit de sécurité
cargo audit
```

### Base de données

```bash
# Créer une migration
sqlx migrate add <name>

# Réinitialiser la DB
sqlx database reset

# Status des migrations
sqlx migrate info
```

## Résolution de problèmes

### Erreur de connexion PostgreSQL

```
Error: could not connect to database
```

**Solutions** :
1. Vérifier que PostgreSQL est démarré
2. Vérifier `DATABASE_URL` dans `.env`
3. Vérifier les credentials

### Erreur de migration

```
Error: migration checksum mismatch
```

**Solution** :

```bash
sqlx database reset
sqlx migrate run
```

### Erreur JWT

```
Error: Failed to load JWT keys
```

**Solution** : Régénérer les clés (voir section 3)

## Structure du projet

```
T-JSF-600-PAR_20/
├── backend/           # Code Rust
│   ├── src/          # Sources
│   ├── migrations/   # Migrations SQL
│   ├── keys/         # Clés JWT (gitignored)
│   ├── Cargo.toml    # Dépendances
│   └── .env          # Configuration (gitignored)
├── docs/             # Documentation
├── frontend/         # (à venir)
└── docker-compose.yml
```

## Prochaines étapes

1. [Documentation API](../api/README.md)
2. [Protocole WebSocket](../websocket/README.md)
3. [Architecture](../architecture/README.md)

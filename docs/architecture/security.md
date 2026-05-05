# Sécurité

## Vue d'ensemble

Ce document décrit les mesures de sécurité implémentées dans le projet Discord Clone.

## Authentification

### JWT (JSON Web Tokens)

**Algorithme** : RS256 (RSA + SHA-256)

**Avantages de RS256 vs HS256** :
- Clé privée reste sur le serveur
- Clé publique peut être distribuée pour vérification
- Révocation plus simple

**Structure des tokens** :

```
Access Token:
├── Header: { "alg": "RS256", "typ": "JWT" }
├── Payload:
│   ├── sub: user_id
│   ├── email: user@example.com
│   ├── username: johndoe
│   ├── type: "access"
│   ├── iat: issued_at
│   └── exp: expires_at (15 min)
└── Signature

Refresh Token:
├── Header: { "alg": "RS256", "typ": "JWT" }
├── Payload:
│   ├── sub: user_id
│   ├── type: "refresh"
│   ├── jti: unique_token_id
│   ├── iat: issued_at
│   └── exp: expires_at (7 days)
└── Signature
```

### Gestion des mots de passe

**Algorithme** : Argon2id (recommandé OWASP)

**Paramètres** :
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4
- Salt: 16 bytes (random)

```rust
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

let argon2 = Argon2::new(
    argon2::Algorithm::Argon2id,
    argon2::Version::V0x13,
    argon2::Params::new(65536, 3, 4, None).unwrap()
);
```

### Politique de mots de passe

| Critère | Exigence |
| ------- | -------- |
| Longueur minimale | 8 caractères |
| Majuscule | Au moins 1 |
| Minuscule | Au moins 1 |
| Chiffre | Au moins 1 |
| Caractère spécial | Au moins 1 |

## Autorisation (RBAC)

### Hiérarchie des rôles

```
owner (4) > admin (3) > moderator (2) > member (1)
```

### Matrice de permissions

| Action | owner | admin | moderator | member |
| ------ | ----- | ----- | --------- | ------ |
| Supprimer serveur | ✅ | ❌ | ❌ | ❌ |
| Transférer propriété | ✅ | ❌ | ❌ | ❌ |
| Promouvoir admin | ✅ | ❌ | ❌ | ❌ |
| Gérer channels | ✅ | ✅ | ❌ | ❌ |
| Gérer membres | ✅ | ✅ | ❌ | ❌ |
| Kick membres | ✅ | ✅ | ✅ | ❌ |
| Supprimer messages | ✅ | ✅ | ✅ | ❌ |
| Créer invitations | ✅ | ✅ | ✅ | ❌ |
| Envoyer messages | ✅ | ✅ | ✅ | ✅ |
| Lire messages | ✅ | ✅ | ✅ | ✅ |

### Règles de modification de rôle

1. Ne peut pas modifier son propre rôle
2. Ne peut pas promouvoir au-dessus de son propre rôle
3. Ne peut pas modifier un rôle égal ou supérieur
4. Seul le owner peut promouvoir en admin

## Protection contre les attaques

### Injection SQL

**Mitigation** : Requêtes préparées avec SQLx

```rust
// ✅ Correct - requête préparée
sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = $1",
    email
)

// ❌ Incorrect - concaténation
format!("SELECT * FROM users WHERE email = '{}'", email)
```

### XSS (Cross-Site Scripting)

**Mitigations** :
- Content sanitization côté serveur
- Headers de sécurité
- CSP (Content Security Policy)

```rust
// Headers de réponse
response.headers_mut().insert(
    "Content-Security-Policy",
    "default-src 'self'; script-src 'self'".parse().unwrap()
);
```

### CSRF (Cross-Site Request Forgery)

**Mitigations** :
- Token-based auth (JWT) plutôt que cookies de session
- Validation de l'origine des requêtes
- SameSite cookies si utilisés

### Rate Limiting

| Endpoint | Limite |
| -------- | ------ |
| /auth/login | 5/min par IP |
| /auth/register | 3/min par IP |
| /api/v1/* | 100/min par user |
| WebSocket messages | 60/min par user |

**Implémentation** :

```rust
use tower_governor::GovernorLayer;

let governor = GovernorLayer::per_second(10);
```

## Headers de sécurité

```rust
// Middleware headers sécurité
app.layer(SetResponseHeaderLayer::overriding(
    header::X_CONTENT_TYPE_OPTIONS,
    HeaderValue::from_static("nosniff"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::X_FRAME_OPTIONS,
    HeaderValue::from_static("DENY"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::STRICT_TRANSPORT_SECURITY,
    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
))
```

## Gestion des secrets

### Variables d'environnement

```bash
# .env (ne jamais commit!)
DATABASE_URL=postgres://user:pass@localhost/db
JWT_PRIVATE_KEY_PATH=/secrets/jwt_private.pem
JWT_PUBLIC_KEY_PATH=/secrets/jwt_public.pem
```

### Génération des clés JWT

```bash
# Générer la clé privée
openssl genrsa -out jwt_private.pem 4096

# Extraire la clé publique
openssl rsa -in jwt_private.pem -pubout -out jwt_public.pem
```

### Rotation des clés

1. Générer nouvelle paire de clés
2. Accepter les deux clés pendant la période de transition
3. Attendre expiration des anciens tokens
4. Retirer l'ancienne clé

## Logging sécurisé

### Données à NE PAS logger

- Mots de passe (même hashés)
- Tokens JWT complets
- Données personnelles (email, IP si possible)
- Contenu des messages

### Format recommandé

```rust
// ✅ Correct
tracing::info!(user_id = %user.id, action = "login", "User logged in");

// ❌ Incorrect
tracing::info!("User {} logged in with password {}", email, password);
```

## Checklist de sécurité

### Avant déploiement

- [ ] Variables d'environnement configurées (pas de secrets en dur)
- [ ] HTTPS activé en production
- [ ] Rate limiting configuré
- [ ] Logs en mode production (pas de debug)
- [ ] Headers de sécurité activés
- [ ] Audit des dépendances (`cargo audit`)

### Audit régulier

```bash
# Vérifier les vulnérabilités des dépendances
cargo audit

# Scanner le code avec clippy en mode strict
cargo clippy -- -D warnings
```

## Conformité RGPD

### Données personnelles collectées

| Donnée | Usage | Rétention |
| ------ | ----- | --------- |
| Email | Authentification | Jusqu'à suppression compte |
| Username | Identification | Jusqu'à suppression compte |
| IP (logs) | Sécurité | 30 jours |
| Messages | Communication | Paramétrable |

### Droits des utilisateurs

- **Accès** : GET /auth/me
- **Rectification** : PUT /users/me
- **Suppression** : DELETE /users/me
- **Export** : GET /users/me/export

## Incident Response

### En cas de breach

1. Isoler les systèmes affectés
2. Révoquer tous les refresh tokens
3. Forcer la réauthentification
4. Analyser les logs
5. Notifier les utilisateurs concernés
6. Corriger la vulnérabilité
7. Post-mortem

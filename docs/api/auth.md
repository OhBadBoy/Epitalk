# API Authentication

## Vue d'ensemble

L'authentification utilise des tokens JWT (JSON Web Tokens) avec l'algorithme RS256.

- **Access Token** : Durée de vie 15 minutes
- **Refresh Token** : Durée de vie 7 jours

## Endpoints

---

### POST /auth/register

Créer un nouveau compte utilisateur.

**Request**

```http
POST /api/v1/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "johndoe",
  "password": "SecureP@ss123"
}
```

**Validation**

| Champ | Règles |
| ----- | ------ |
| email | Format email valide, unique |
| username | 3-32 caractères, alphanumérique + underscore |
| password | Min 8 caractères, 1 majuscule, 1 chiffre, 1 spécial |

**Response 201 Created**

```json
{
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "username": "johndoe",
      "avatar_url": null,
      "created_at": "2026-02-02T10:00:00Z"
    },
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Invalid email format |
| 400 | Password too weak |
| 409 | Email already registered |
| 409 | Username already taken |

---

### POST /auth/login

Authentifier un utilisateur existant.

**Request**

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecureP@ss123"
}
```

**Response 200 OK**

```json
{
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "username": "johndoe",
      "avatar_url": "https://cdn.example.com/avatars/123.png",
      "created_at": "2026-02-02T10:00:00Z"
    },
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 401 | Invalid credentials |
| 429 | Too many login attempts |

---

### GET /auth/me

Récupérer le profil de l'utilisateur connecté.

**Request**

```http
GET /api/v1/auth/me
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "avatar_url": "https://cdn.example.com/avatars/123.png",
    "created_at": "2026-02-02T10:00:00Z",
    "servers": [
      {
        "id": "660e8400-e29b-41d4-a716-446655440001",
        "name": "My Server",
        "role": "owner"
      }
    ]
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 401 | Token expired |
| 401 | Invalid token |

---

### POST /auth/refresh

Obtenir un nouveau access token avec le refresh token.

**Request**

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response 200 OK**

```json
{
  "data": {
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 401 | Invalid refresh token |
| 401 | Refresh token expired |
| 401 | Token has been revoked |

---

## JWT Structure

### Access Token Payload

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "username": "johndoe",
  "type": "access",
  "iat": 1706868000,
  "exp": 1706868900
}
```

### Refresh Token Payload

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "type": "refresh",
  "jti": "unique-token-id",
  "iat": 1706868000,
  "exp": 1707472800
}
```

## Sécurité

- Les mots de passe sont hashés avec **Argon2id**
- Les tokens utilisent **RS256** (asymétrique)
- Rate limiting sur `/auth/*` : 10 requêtes/minute
- Refresh tokens révocables (stockés en base)
- Protection CSRF pour les cookies (si utilisés)

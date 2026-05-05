# API REST - Vue d'ensemble

## Base URL

```
https://api.discord-clone.example.com/api/v1
```

En développement local :
```
http://localhost:3001/api/v1
```

## Authentification

Toutes les routes (sauf `/auth/register` et `/auth/login`) nécessitent un token JWT.

```http
Authorization: Bearer <access_token>
```

## Format des réponses

### Succès

```json
{
  "data": { ... },
  "meta": {
    "timestamp": "2026-02-02T10:00:00Z"
  }
}
```

### Erreur

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid email format",
    "details": { ... }
  }
}
```

## Codes HTTP

| Code | Description |
| ---- | ----------- |
| 200 | OK - Requête réussie |
| 201 | Created - Ressource créée |
| 204 | No Content - Suppression réussie |
| 400 | Bad Request - Données invalides |
| 401 | Unauthorized - Token manquant/invalide |
| 403 | Forbidden - Permissions insuffisantes |
| 404 | Not Found - Ressource inexistante |
| 409 | Conflict - Ressource déjà existante |
| 422 | Unprocessable Entity - Validation échouée |
| 429 | Too Many Requests - Rate limit atteint |
| 500 | Internal Server Error |

## Endpoints

### Authentication

| Méthode | Endpoint | Description |
| ------- | -------- | ----------- |
| POST | `/auth/register` | Créer un compte |
| POST | `/auth/login` | Se connecter |
| GET | `/auth/me` | Profil utilisateur |
| POST | `/auth/refresh` | Rafraîchir le token |

→ [Documentation détaillée](./auth.md)

### Servers

| Méthode | Endpoint | Description |
| ------- | -------- | ----------- |
| GET | `/servers` | Liste des serveurs |
| POST | `/servers` | Créer un serveur |
| GET | `/servers/:id` | Détails d'un serveur |
| PUT | `/servers/:id` | Modifier un serveur |
| DELETE | `/servers/:id` | Supprimer un serveur |

→ [Documentation détaillée](./servers.md)

### Channels

| Méthode | Endpoint | Description |
| ------- | -------- | ----------- |
| GET | `/servers/:server_id/channels` | Liste des channels |
| POST | `/servers/:server_id/channels` | Créer un channel |
| GET | `/channels/:id` | Détails d'un channel |
| PUT | `/channels/:id` | Modifier un channel |
| DELETE | `/channels/:id` | Supprimer un channel |

→ [Documentation détaillée](./channels.md)

### Members

| Méthode | Endpoint | Description |
| ------- | -------- | ----------- |
| GET | `/servers/:server_id/members` | Liste des membres |
| GET | `/servers/:server_id/members/:user_id` | Détails d'un membre |
| PUT | `/servers/:server_id/members/:user_id` | Modifier le rôle |
| DELETE | `/servers/:server_id/members/:user_id` | Retirer un membre |

→ [Documentation détaillée](./members.md)

### Invites

| Méthode | Endpoint | Description |
| ------- | -------- | ----------- |
| GET | `/servers/:server_id/invites` | Liste des invitations |
| POST | `/servers/:server_id/invites` | Créer une invitation |
| POST | `/invites/:code/join` | Rejoindre via invitation |
| DELETE | `/invites/:code` | Supprimer une invitation |

→ [Documentation détaillée](./invites.md)

## Rate Limiting

| Endpoint | Limite |
| -------- | ------ |
| `/auth/*` | 10 req/min |
| `/api/v1/*` | 100 req/min |
| WebSocket | 60 msg/min |

Headers de réponse :
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1706871600
```

## Pagination

Pour les listes, utiliser les query params :

```http
GET /servers?page=1&per_page=20
```

Réponse avec métadonnées :

```json
{
  "data": [...],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

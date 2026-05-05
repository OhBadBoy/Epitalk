# API Invites

## Vue d'ensemble

Les invitations permettent aux utilisateurs de rejoindre un serveur via un code unique. Les invitations peuvent être permanentes ou expirables.

## Endpoints

### GET /servers/:server_id/invites

Liste des invitations d'un serveur. Requiert le rôle `moderator` ou supérieur.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/invites
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440001",
      "code": "abc123XY",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "created_by": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "johndoe"
      },
      "uses": 5,
      "max_uses": 10,
      "expires_at": "2026-02-09T10:00:00Z",
      "created_at": "2026-02-02T10:00:00Z"
    },
    {
      "id": "880e8400-e29b-41d4-a716-446655440002",
      "code": "xyz789AB",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "created_by": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "janedoe"
      },
      "uses": 0,
      "max_uses": null,
      "expires_at": null,
      "created_at": "2026-02-01T15:30:00Z"
    }
  ]
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Insufficient permissions |
| 404 | Server not found |

### GET /servers/:server_id/invites/active

Liste des invitations **actives** d'un serveur (non expirées et n'ayant pas atteint le nombre max d'utilisations). Requiert le rôle `moderator` ou supérieur.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/invites/active
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440002",
      "code": "xyz789AB",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "created_by": "550e8400-e29b-41d4-a716-446655440001",
      "uses": 0,
      "max_uses": null,
      "expires_at": null,
      "created_at": "2026-02-01T15:30:00Z",
      "is_valid": true
    }
  ]
}
```

**Différence avec GET /invites**

| Endpoint | Retourne |
| -------- | -------- |
| `GET /invites` | Toutes les invitations (y compris expirées) |
| `GET /invites/active` | Uniquement les invitations utilisables |

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Insufficient permissions |
| 404 | Server not found |

Créer une invitation. Requiert le rôle `moderator` ou supérieur.

**Request**

```http
POST /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/invites
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "max_uses": 10,
  "expires_in_hours": 168
}
```

**Validation**

| Champ | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| max_uses | integer | null | Nombre max d'utilisations (null = illimité) |
| expires_in_hours | integer | null | Expiration en heures (null = permanent) |

**Presets suggérés**

| Preset | max_uses | expires_in_hours |
| ------ | -------- | ---------------- |
| Temporaire | 1 | 1 |
| Standard | 10 | 168 (7 jours) |
| Permanent | null | null |

**Response 201 Created**

```json
{
  "data": {
    "id": "880e8400-e29b-41d4-a716-446655440003",
    "code": "newCode1",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "created_by": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "johndoe"
    },
    "uses": 0,
    "max_uses": 10,
    "expires_at": "2026-02-09T10:00:00Z",
    "created_at": "2026-02-02T10:00:00Z",
    "invite_url": "https://discord-clone.example.com/invite/newCode1"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | max_uses must be positive |
| 400 | expires_in_hours must be positive |
| 403 | Insufficient permissions |
| 404 | Server not found |
| 429 | Too many invites created |

### GET /invites/:code

Obtenir les informations d'une invitation (public).

**Request**

```http
GET /api/v1/invites/abc123XY
```

**Note** : Cet endpoint est public et ne nécessite pas d'authentification.

**Response 200 OK**

```json
{
  "data": {
    "code": "abc123XY",
    "server": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "Gaming Squad",
      "icon_url": "https://cdn.example.com/icons/gaming.png",
      "member_count": 42
    },
    "created_by": {
      "username": "johndoe",
      "avatar_url": "https://cdn.example.com/avatars/123.png"
    },
    "expires_at": "2026-02-09T10:00:00Z"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 404 | Invite not found |
| 410 | Invite has expired |
| 410 | Invite has reached max uses |

### POST /invites/:code/join

Rejoindre un serveur via une invitation.

**Request**

```http
POST /api/v1/invites/abc123XY/join
Authorization: Bearer <access_token>
```

**Response 201 Created**

```json
{
  "data": {
    "server": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "Gaming Squad",
      "icon_url": "https://cdn.example.com/icons/gaming.png",
      "member_count": 43
    },
    "membership": {
      "user_id": "550e8400-e29b-41d4-a716-446655440002",
      "role": "member",
      "joined_at": "2026-02-02T10:00:00Z"
    }
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 404 | Invite not found |
| 409 | Already a member of this server |
| 410 | Invite has expired |
| 410 | Invite has reached max uses |

### DELETE /invites/:code

Supprimer une invitation. Requiert le rôle `moderator` ou supérieur, ou être le créateur.

**Request**

```http
DELETE /api/v1/invites/abc123XY
Authorization: Bearer <access_token>
```

**Response 204 No Content**

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Insufficient permissions |
| 404 | Invite not found |

## Structure du code d'invitation

Les codes d'invitation sont générés avec les caractéristiques suivantes :

- Longueur : 8 caractères
- Caractères : A-Z, a-z, 0-9 (base62)
- Unicité garantie
- Case-insensitive pour la recherche

**Exemple** : `abc123XY`, `XyZ789Ab`

## Événements WebSocket associés

| Événement | Description |
| --------- | ----------- |
| `invite.created` | Nouvelle invitation créée |
| `invite.deleted` | Invitation supprimée |
| `invite.used` | Invitation utilisée (membre rejoint) |

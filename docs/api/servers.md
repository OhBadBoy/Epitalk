# API Servers

## Vue d'ensemble

Les serveurs sont les espaces de communication principaux. Chaque serveur contient des channels et des membres avec différents rôles.

## Rôles

| Rôle | Permissions |
| ---- | ----------- |
| owner | Toutes les permissions, transfert de propriété |
| admin | Gérer channels, membres, invitations |
| moderator | Modérer messages, kick membres |
| member | Lire/écrire messages |

## Endpoints

### GET /servers

Liste des serveurs dont l'utilisateur est membre.

**Request**

```http
GET /api/v1/servers
Authorization: Bearer <access_token>
```

**Query Parameters**

| Param | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| page | integer | 1 | Numéro de page |
| per_page | integer | 20 | Éléments par page (max 100) |

**Response 200 OK**

```json
{
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "Gaming Squad",
      "description": "Server for gamers",
      "icon_url": "https://cdn.example.com/icons/gaming.png",
      "owner_id": "550e8400-e29b-41d4-a716-446655440000",
      "member_count": 42,
      "my_role": "admin",
      "created_at": "2026-01-15T08:30:00Z"
    }
  ],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 5,
    "total_pages": 1
  }
}
```

### POST /servers

Créer un nouveau serveur. L'utilisateur devient owner automatiquement.

**Request**

```http
POST /api/v1/servers
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "My Awesome Server",
  "description": "A place for friends"
}
```

**Validation**

| Champ | Règles |
| ----- | ------ |
| name | Requis, 2-100 caractères |
| description | Optionnel, max 1000 caractères |

**Response 201 Created**

```json
{
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440002",
    "name": "My Awesome Server",
    "description": "A place for friends",
    "icon_url": null,
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "member_count": 1,
    "my_role": "owner",
    "created_at": "2026-02-02T10:00:00Z"
  }
}
```

**Note** : Un channel `#general` est créé automatiquement.

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Name is required |
| 400 | Name must be between 2 and 100 characters |
| 429 | Server creation rate limit exceeded |

### GET /servers/:id

Détails d'un serveur spécifique.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Gaming Squad",
    "description": "Server for gamers",
    "icon_url": "https://cdn.example.com/icons/gaming.png",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "member_count": 42,
    "my_role": "admin",
    "created_at": "2026-01-15T08:30:00Z",
    "channels": [
      {
        "id": "770e8400-e29b-41d4-a716-446655440001",
        "name": "general",
        "channel_type": "text"
      },
      {
        "id": "770e8400-e29b-41d4-a716-446655440002",
        "name": "voice-chat",
        "channel_type": "voice"
      }
    ]
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Not a member of this server |
| 404 | Server not found |

### PUT /servers/:id

Modifier un serveur. Requiert le rôle `admin` ou `owner`.

**Request**

```http
PUT /api/v1/servers/660e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "Gaming Squad Pro",
  "description": "Elite gamers only",
  "icon_url": "https://cdn.example.com/icons/gaming-pro.png"
}
```

**Response 200 OK**

```json
{
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Gaming Squad Pro",
    "description": "Elite gamers only",
    "icon_url": "https://cdn.example.com/icons/gaming-pro.png",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "member_count": 42,
    "my_role": "admin",
    "created_at": "2026-01-15T08:30:00Z"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Insufficient permissions |
| 404 | Server not found |

### DELETE /servers/:id

Supprimer un serveur. Requiert le rôle `owner`.

**Request**

```http
DELETE /api/v1/servers/660e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Response 204 No Content**

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Only the owner can delete a server |
| 404 | Server not found |

## Événements WebSocket associés

| Événement | Description |
| --------- | ----------- |
| `server.created` | Nouveau serveur créé |
| `server.updated` | Serveur modifié |
| `server.deleted` | Serveur supprimé |
| `server.member_joined` | Nouveau membre |
| `server.member_left` | Membre parti |

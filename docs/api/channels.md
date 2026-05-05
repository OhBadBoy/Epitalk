# API Channels

## Vue d'ensemble

Les channels sont les espaces de discussion au sein d'un serveur. Il existe deux types : `text` pour les messages et `voice` pour la communication vocale.

## Types de channels

| Type | Description |
| ---- | ----------- |
| text | Messages texte, images, fichiers |
| voice | Communication audio/vidéo en temps réel |

## Endpoints

### GET /servers/:server_id/channels

Liste des channels d'un serveur.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/channels
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440001",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "general",
      "description": "General discussion",
      "channel_type": "text",
      "position": 0,
      "created_at": "2026-01-15T08:30:00Z"
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "voice-chat",
      "description": null,
      "channel_type": "voice",
      "position": 1,
      "created_at": "2026-01-15T08:35:00Z"
    }
  ]
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Not a member of this server |
| 404 | Server not found |

### POST /servers/:server_id/channels

Créer un nouveau channel. Requiert le rôle `admin` ou `owner`.

**Request**

```http
POST /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/channels
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "announcements",
  "description": "Important announcements only",
  "channel_type": "text"
}
```

**Validation**

| Champ | Règles |
| ----- | ------ |
| name | Requis, 1-100 caractères, lowercase, pas d'espaces |
| description | Optionnel, max 1024 caractères |
| channel_type | Requis, `text` ou `voice` |

**Response 201 Created**

```json
{
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440003",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "announcements",
    "description": "Important announcements only",
    "channel_type": "text",
    "position": 2,
    "created_at": "2026-02-02T10:00:00Z"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Invalid channel name format |
| 400 | Invalid channel type |
| 403 | Insufficient permissions |
| 404 | Server not found |
| 409 | Channel name already exists |

### GET /channels/:id

Détails d'un channel spécifique.

**Request**

```http
GET /api/v1/channels/770e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440001",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "general",
    "description": "General discussion",
    "channel_type": "text",
    "position": 0,
    "created_at": "2026-01-15T08:30:00Z",
    "last_message_at": "2026-02-02T09:55:00Z"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Not a member of this server |
| 404 | Channel not found |

### PUT /channels/:id

Modifier un channel. Requiert le rôle `admin` ou `owner`.

**Request**

```http
PUT /api/v1/channels/770e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "general-chat",
  "description": "Main chat room"
}
```

**Response 200 OK**

```json
{
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440001",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "general-chat",
    "description": "Main chat room",
    "channel_type": "text",
    "position": 0,
    "created_at": "2026-01-15T08:30:00Z"
  }
}
```

**Note** : Le `channel_type` ne peut pas être modifié après création.

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Insufficient permissions |
| 404 | Channel not found |
| 409 | Channel name already exists |

### DELETE /channels/:id

Supprimer un channel. Requiert le rôle `admin` ou `owner`.

**Request**

```http
DELETE /api/v1/channels/770e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Response 204 No Content**

**Note** : Le dernier channel `text` d'un serveur ne peut pas être supprimé.

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Cannot delete last text channel |
| 403 | Insufficient permissions |
| 404 | Channel not found |

## Événements WebSocket associés

| Événement | Description |
| --------- | ----------- |
| `channel.created` | Nouveau channel créé |
| `channel.updated` | Channel modifié |
| `channel.deleted` | Channel supprimé |
| `channel.typing` | Utilisateur en train d'écrire |

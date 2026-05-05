# API Members

## Vue d'ensemble

Les membres représentent les utilisateurs au sein d'un serveur avec leurs rôles et permissions associées.

## Rôles et hiérarchie

| Niveau | Rôle | Description |
| ------ | ---- | ----------- |
| 4 | owner | Propriétaire du serveur |
| 3 | admin | Administrateur avec pleins pouvoirs |
| 2 | moderator | Modérateur avec pouvoirs limités |
| 1 | member | Membre standard |

## Permissions par rôle

| Permission | owner | admin | moderator | member |
| ---------- | ----- | ----- | --------- | ------ |
| Supprimer serveur | ✅ | ❌ | ❌ | ❌ |
| Gérer rôles | ✅ | ✅ | ❌ | ❌ |
| Gérer channels | ✅ | ✅ | ❌ | ❌ |
| Kick membres | ✅ | ✅ | ✅ | ❌ |
| Supprimer messages | ✅ | ✅ | ✅ | ❌ |
| Créer invitations | ✅ | ✅ | ✅ | ❌ |
| Envoyer messages | ✅ | ✅ | ✅ | ✅ |

## Endpoints

### GET /servers/:server_id/members

Liste des membres d'un serveur.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/members
Authorization: Bearer <access_token>
```

**Query Parameters**

| Param | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| page | integer | 1 | Numéro de page |
| per_page | integer | 50 | Éléments par page (max 100) |
| role | string | - | Filtrer par rôle |

**Response 200 OK**

```json
{
  "data": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "username": "johndoe",
      "avatar_url": "https://cdn.example.com/avatars/123.png",
      "role": "owner",
      "joined_at": "2026-01-15T08:30:00Z"
    },
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440001",
      "server_id": "660e8400-e29b-41d4-a716-446655440001",
      "username": "janedoe",
      "avatar_url": null,
      "role": "admin",
      "joined_at": "2026-01-16T10:00:00Z"
    }
  ],
  "meta": {
    "page": 1,
    "per_page": 50,
    "total": 42,
    "total_pages": 1
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Not a member of this server |
| 404 | Server not found |

### GET /servers/:server_id/members/:user_id

Détails d'un membre spécifique.

**Request**

```http
GET /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Response 200 OK**

```json
{
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440001",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "username": "janedoe",
    "email": "jane@example.com",
    "avatar_url": null,
    "role": "admin",
    "joined_at": "2026-01-16T10:00:00Z",
    "permissions": [
      "manage_channels",
      "manage_members",
      "kick_members",
      "create_invites",
      "send_messages",
      "delete_messages"
    ]
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 403 | Not a member of this server |
| 404 | Server not found |
| 404 | Member not found |

### PUT /servers/:server_id/members/:user_id

Modifier le rôle d'un membre. Requiert un rôle supérieur au membre cible.

**Request**

```http
PUT /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "role": "moderator"
}
```

**Validation**

| Champ | Règles |
| ----- | ------ |
| role | `admin`, `moderator`, ou `member` |

**Contraintes**
- Ne peut pas modifier son propre rôle
- Ne peut pas promouvoir au-dessus de son propre rôle
- Ne peut pas modifier le owner
- Seul le owner peut promouvoir en admin

**Response 200 OK**

```json
{
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440001",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "username": "janedoe",
    "avatar_url": null,
    "role": "moderator",
    "joined_at": "2026-01-16T10:00:00Z"
  }
}
```

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Invalid role |
| 400 | Cannot modify your own role |
| 403 | Cannot modify owner role |
| 403 | Cannot promote above your role |
| 403 | Insufficient permissions |
| 404 | Server not found |
| 404 | Member not found |

### DELETE /servers/:server_id/members/:user_id

Retirer un membre du serveur (kick). Requiert le rôle `moderator` ou supérieur.

**Request**

```http
DELETE /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440001
Authorization: Bearer <access_token>
```

**Contraintes**
- Ne peut pas se kick soi-même (utiliser Leave Server)
- Ne peut pas kick un membre avec un rôle égal ou supérieur
- Ne peut pas kick le owner

**Response 204 No Content**

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Cannot kick yourself |
| 403 | Cannot kick owner |
| 403 | Cannot kick member with equal or higher role |
| 403 | Insufficient permissions |
| 404 | Server not found |
| 404 | Member not found |

### DELETE /servers/:server_id/members/me

Quitter un serveur volontairement.

**Request**

```http
DELETE /api/v1/servers/660e8400-e29b-41d4-a716-446655440001/members/me
Authorization: Bearer <access_token>
```

**Contraintes**
- Le owner ne peut pas quitter sans transférer la propriété

**Response 204 No Content**

**Errors**

| Code | Message |
| ---- | ------- |
| 400 | Owner cannot leave server without transferring ownership |
| 404 | Server not found |

## Événements WebSocket associés

| Événement | Description |
| --------- | ----------- |
| `member.joined` | Nouveau membre a rejoint |
| `member.left` | Membre a quitté |
| `member.kicked` | Membre a été expulsé |
| `member.role_updated` | Rôle d'un membre modifié |
| `member.presence_updated` | Statut en ligne/hors ligne |

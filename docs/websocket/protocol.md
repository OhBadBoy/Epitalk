# Protocole WebSocket - Spécification Complète

## 1. Messages Client → Serveur

### 1.1 Identify (op=2)

Envoyé après réception du Hello pour s'identifier.

```json
{
  "op": 2,
  "d": {
    "token": "eyJhbGciOiJSUzI1NiIs...",
    "properties": {
      "os": "Windows",
      "browser": "Chrome",
      "device": "desktop"
    }
  }
}
```

### 1.2 Heartbeat (op=1)

Ping périodique pour maintenir la connexion.

```json
{
  "op": 1,
  "d": {
    "seq": 42
  }
}
```

### 1.3 Presence Update (op=3)

Mettre à jour son statut de présence.

```json
{
  "op": 3,
  "d": {
    "status": "online",
    "activity": {
      "type": "playing",
      "name": "Visual Studio Code"
    }
  }
}
```

**Statuts disponibles**

| Status | Description |
| ------ | ----------- |
| online | En ligne |
| idle | Inactif |
| dnd | Ne pas déranger |
| invisible | Invisible (apparaît hors ligne) |

### 1.4 Subscribe (op=4)

S'abonner aux événements d'un channel.

```json
{
  "op": 4,
  "d": {
    "channel_id": "770e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 1.5 Unsubscribe (op=5)

Se désabonner d'un channel.

```json
{
  "op": 5,
  "d": {
    "channel_id": "770e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 1.6 Message Send (op=6)

Envoyer un message dans un channel.

```json
{
  "op": 6,
  "d": {
    "channel_id": "770e8400-e29b-41d4-a716-446655440001",
    "content": "Hello, World!",
    "nonce": "unique-client-id-123"
  }
}
```

**Champs optionnels**

| Champ | Type | Description |
| ----- | ---- | ----------- |
| reply_to | uuid | ID du message auquel répondre |
| attachments | array | Fichiers joints (URLs) |

### 1.7 Typing Start (op=7)

Indiquer que l'utilisateur est en train d'écrire.

```json
{
  "op": 7,
  "d": {
    "channel_id": "770e8400-e29b-41d4-a716-446655440001"
  }
}
```

## 2. Messages Serveur → Client

### 2.1 Hello (op=10)

Premier message après connexion.

```json
{
  "op": 10,
  "d": {
    "heartbeat_interval": 30000,
    "session_id": "abc123"
  }
}
```

### 2.2 HeartbeatAck (op=11)

Accusé de réception du heartbeat.

```json
{
  "op": 11,
  "d": {
    "ack": true
  }
}
```

### 2.3 Dispatch (op=0)

Événements du serveur. Le champ `t` indique le type.

```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 1,
  "d": { ... }
}
```

## 3. Événements (Dispatch)

### 3.1 READY

Confirmation de connexion réussie.

```json
{
  "op": 0,
  "t": "READY",
  "s": 1,
  "d": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "johndoe",
      "avatar_url": "https://cdn.example.com/avatars/123.png"
    },
    "servers": [
      {
        "id": "660e8400-e29b-41d4-a716-446655440001",
        "name": "Gaming Squad"
      }
    ],
    "session_id": "abc123"
  }
}
```

### 3.2 MESSAGE_CREATE

Nouveau message reçu.

```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 2,
  "d": {
    "id": "msg-uuid",
    "channel_id": "770e8400-e29b-41d4-a716-446655440001",
    "author": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "janedoe",
      "avatar_url": null
    },
    "content": "Hello everyone!",
    "timestamp": "2026-02-02T10:00:00Z",
    "edited_timestamp": null,
    "attachments": [],
    "mentions": [],
    "nonce": "unique-client-id-123"
  }
}
```

### 3.3 MESSAGE_UPDATE

Message modifié.

```json
{
  "op": 0,
  "t": "MESSAGE_UPDATE",
  "s": 3,
  "d": {
    "id": "msg-uuid",
    "channel_id": "770e8400-e29b-41d4-a716-446655440001",
    "content": "Hello everyone! (edited)",
    "edited_timestamp": "2026-02-02T10:05:00Z"
  }
}
```

### 3.4 MESSAGE_DELETE

Message supprimé.

```json
{
  "op": 0,
  "t": "MESSAGE_DELETE",
  "s": 4,
  "d": {
    "id": "msg-uuid",
    "channel_id": "770e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 3.5 TYPING_START

Utilisateur en train d'écrire (expire après 10s).

```json
{
  "op": 0,
  "t": "TYPING_START",
  "s": 5,
  "d": {
    "channel_id": "770e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440001",
    "username": "janedoe",
    "timestamp": "2026-02-02T10:00:00Z"
  }
}
```

### 3.6 PRESENCE_UPDATE

Changement de statut d'un utilisateur.

```json
{
  "op": 0,
  "t": "PRESENCE_UPDATE",
  "s": 6,
  "d": {
    "user_id": "550e8400-e29b-41d4-a716-446655440001",
    "status": "idle",
    "activity": null
  }
}
```

### 3.7 CHANNEL_CREATE

Nouveau channel créé.

```json
{
  "op": 0,
  "t": "CHANNEL_CREATE",
  "s": 7,
  "d": {
    "id": "770e8400-e29b-41d4-a716-446655440003",
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "new-channel",
    "channel_type": "text",
    "position": 2
  }
}
```

### 3.8 CHANNEL_UPDATE

Channel modifié.

```json
{
  "op": 0,
  "t": "CHANNEL_UPDATE",
  "s": 8,
  "d": {
    "id": "770e8400-e29b-41d4-a716-446655440001",
    "name": "general-renamed",
    "description": "New description"
  }
}
```

### 3.9 CHANNEL_DELETE

Channel supprimé.

```json
{
  "op": 0,
  "t": "CHANNEL_DELETE",
  "s": 9,
  "d": {
    "id": "770e8400-e29b-41d4-a716-446655440003",
    "server_id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 3.10 MEMBER_JOIN

Nouveau membre sur un serveur.

```json
{
  "op": 0,
  "t": "MEMBER_JOIN",
  "s": 10,
  "d": {
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "newuser",
      "avatar_url": null
    },
    "joined_at": "2026-02-02T10:00:00Z"
  }
}
```

### 3.11 MEMBER_LEAVE

Membre a quitté ou été expulsé.

```json
{
  "op": 0,
  "t": "MEMBER_LEAVE",
  "s": 11,
  "d": {
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440002",
    "kicked": false
  }
}
```

### 3.12 MEMBER_UPDATE

Rôle d'un membre modifié.

```json
{
  "op": 0,
  "t": "MEMBER_UPDATE",
  "s": 12,
  "d": {
    "server_id": "660e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440002",
    "role": "moderator"
  }
}
```

### 3.13 SERVER_UPDATE

Serveur modifié.

```json
{
  "op": 0,
  "t": "SERVER_UPDATE",
  "s": 13,
  "d": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Gaming Squad Pro",
    "description": "Updated description",
    "icon_url": "https://cdn.example.com/icons/new.png"
  }
}
```

### 3.14 SERVER_DELETE

Serveur supprimé.

```json
{
  "op": 0,
  "t": "SERVER_DELETE",
  "s": 14,
  "d": {
    "id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

## 4. Codes d'erreur WebSocket

### Codes de fermeture

| Code | Nom | Description |
| ---- | --- | ----------- |
| 4000 | Unknown Error | Erreur inconnue |
| 4001 | Unknown Opcode | Code opération invalide |
| 4002 | Decode Error | Impossible de décoder le message |
| 4003 | Not Authenticated | Identify non envoyé |
| 4004 | Authentication Failed | Token invalide |
| 4005 | Already Authenticated | Double Identify |
| 4006 | Invalid Seq | Numéro de séquence invalide |
| 4007 | Rate Limited | Trop de messages |
| 4008 | Session Timeout | Session expirée |
| 4009 | Invalid Session | Session invalide |

### Exemple de gestion d'erreur

```javascript
ws.onclose = (event) => {
  switch (event.code) {
    case 4004:
      // Token invalide, rediriger vers login
      window.location.href = '/login';
      break;
    case 4007:
      // Rate limited, attendre avant reconnexion
      setTimeout(() => reconnect(), 60000);
      break;
    default:
      // Reconnexion avec backoff exponentiel
      reconnectWithBackoff();
  }
};
```

## 5. Rate Limiting

| Type | Limite |
| ---- | ------ |
| Messages | 5/5s par channel |
| Typing | 1/5s par channel |
| Presence | 1/10s |
| Global | 120/min |

## 6. Bonnes pratiques

1. **Toujours gérer les heartbeats** - Sans heartbeat, la connexion sera fermée
2. **Implémenter la reconnexion** - Avec backoff exponentiel
3. **Utiliser des nonces** - Pour corréler les messages envoyés/reçus
4. **Respecter le rate limiting** - Implémenter un queue côté client
5. **Valider les données** - Ne pas faire confiance aux données reçues

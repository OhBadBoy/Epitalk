# WebSocket - Vue d'ensemble (implementation actuelle)

Ce document decrit le protocole WebSocket effectivement implemente par le backend Rust et consomme par le frontend Next.js.

## Endpoint

- Developpement: `ws://localhost:3001/ws`
- Production: definir l'URL via variable frontend `NEXT_PUBLIC_WS_URL`

Connexion avec JWT en query param:

```text
ws://localhost:3001/ws?token=<jwt>
```

Le token est valide au handshake. Si invalide/expire, le serveur refuse la connexion.

## Format des messages

Le protocole n'utilise pas d'opcodes `op/t/d`.

Tous les messages utilisent un format type-safe:

```json
{
  "type": "EventName",
  "payload": {}
}
```

## Client -> Serveur

```json
{ "type": "MessageSend", "payload": { "channel_id": "uuid", "content": "Hello" } }
{ "type": "JoinChannel", "payload": { "channel_id": "uuid" } }
{ "type": "LeaveChannel", "payload": { "channel_id": "uuid" } }
{ "type": "TypingStart", "payload": { "channel_id": "uuid" } }
{ "type": "TypingStop", "payload": { "channel_id": "uuid" } }
{ "type": "Ping" }
```

## Serveur -> Client

```json
{ "type": "MessageNew", "payload": { "id": "...", "channel_id": "...", "author_id": "...", "username": "...", "content": "...", "created_at": "..." } }
{ "type": "MessageUpdated", "payload": { "id": "...", "channel_id": "...", "author_id": "...", "username": "...", "content": "...", "edited_at": "..." } }
{ "type": "MessagePinned", "payload": { "message_id": "...", "channel_id": "...", "pinned_by": "...", "pinned_at": "..." } }
{ "type": "MessageUnpinned", "payload": { "message_id": "...", "channel_id": "...", "unpinned_by": "...", "unpinned_at": "..." } }
{ "type": "UserJoined", "payload": { "user_id": "...", "channel_id": "..." } }
{ "type": "UserLeft", "payload": { "user_id": "...", "channel_id": "..." } }
{ "type": "TypingStart", "payload": { "user_id": "...", "username": "...", "channel_id": "..." } }
{ "type": "TypingStop", "payload": { "user_id": "...", "username": "...", "channel_id": "..." } }
{ "type": "UserOnline", "payload": { "user_id": "..." } }
{ "type": "UserOffline", "payload": { "user_id": "..." } }
{ "type": "Pong" }
{ "type": "Error", "payload": { "code": "...", "message": "..." } }
```

## Comportement temps reel

- Le client envoie `Ping` periodiquement; le serveur repond `Pong`.
- `JoinChannel` ajoute la connexion a la room et envoie l'historique recent.
- `TypingStart` est throttle cote serveur.
- Les evenements `UserOnline/UserOffline` sont diffuses globalement.

## Reconnexion frontend

Le frontend implemente:

- reconnexion automatique avec backoff
- mode degrade apres plusieurs echecs reseau
- logout uniquement sur invalidation auth confirmee

Ces regles sont gerees dans `frontend/real-time-chat/store/websocket.store.ts`.

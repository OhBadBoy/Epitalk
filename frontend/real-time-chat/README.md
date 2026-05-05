# EpiTalk Frontend

Application Next.js du client EpiTalk (chat temps reel).

## Prerequis

- Node.js 20+
- Backend EpiTalk accessible (HTTP + WebSocket)

## Variables d'environnement

Creer ou mettre a jour `frontend/real-time-chat/.env.local` :

```bash
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws
```

- `NEXT_PUBLIC_API_URL` : base URL HTTP backend (les appels utilisent `${NEXT_PUBLIC_API_URL}/api`)
- `NEXT_PUBLIC_WS_URL` : endpoint WebSocket complet (token JWT ajoute en query param)

## Lancer en developpement

```bash
npm install
npm run dev
```

Frontend disponible sur `http://localhost:3000`.

## Build de validation

```bash
npm run build
```

## Notes WebSocket

- Connexion vers `NEXT_PUBLIC_WS_URL?token=<jwt>`
- Reconnexion automatique avec backoff
- Mode degrade apres plusieurs echecs reseau
- Deconnexion utilisateur seulement en cas de session invalide confirmee

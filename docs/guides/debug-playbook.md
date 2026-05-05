# Guide De Debug - EpiTalk

## Objectif
Ce document sert de guide rapide pour diagnostiquer les incidents les plus frequents sur le projet (backend, frontend, desktop, CI) et lister les bugs potentiels connus.

## 1) Checklist Demarrage Rapide

### Backend
1. Se placer dans `backend`.
2. Verifier que Docker DB tourne:
   - `docker compose ps`
3. Verifier `backend/.env`:
   - `DATABASE_URL` doit pointer vers `localhost:5433` en local.
4. Lancer:
   - `EPITALK_DISABLE_SUPERVISOR=1 cargo run`

Symptome critique:
- `pool timed out while waiting for an open connection`
Cause probable:
- `.env` absent (fallback sur 5432) ou Postgres non demarre.

### Frontend
1. Se placer dans `frontend/real-time-chat`.
2. Installer deps:
   - `npm install --legacy-peer-deps` (conflit peer React 19 / emoji-mart)
3. Lancer:
   - `npm run dev`

Symptome critique:
- `Unable to reach the server`
Cause probable:
- backend down, mauvais `NEXT_PUBLIC_API_URL`, CORS, token invalide.

### Desktop
1. Se placer dans `desktop-app`.
2. Installer deps:
   - `npm install`
3. Lancer:
   - `npm start`

## 2) Scenarios De Panne Courants

### A. Erreur `NOT_MEMBER` en chat
Symptome:
- Console web: `You are not a member of this server`.

Diagnostic:
1. Le token est valide, mais l'utilisateur n'est pas membre du serveur actif.
2. `server-store`/`channel-store` peuvent garder un ancien serveur actif.

Resolution:
1. Rejoindre un serveur via invite.
2. Purger les stores si necessaire:
   - `localStorage.removeItem("server-store")`
   - `localStorage.removeItem("channel-store")`
   - `location.reload()`

### B. Connexion backend en boucle supervisor
Symptome:
- `[supervisor] backend crashed ... restarting`.

Diagnostic:
- Le process enfant crash (souvent DB URL incorrecte).

Resolution:
- Lancer sans superviseur pour diagnostiquer la vraie erreur:
  - `EPITALK_DISABLE_SUPERVISOR=1 cargo run`

### C. CI ne se declenche pas sur feature branch
Symptome:
- Aucun run CI complet apres push sur branche feature.

Cause:
- Workflow CI configure pour `push`/`pull_request` vers `main` et `develop`.

Resolution:
- Ouvrir une PR vers `main` ou `develop`.

## 3) Points A Observer Dans Les Logs

### Backend
- `Configuration loaded`
- `PostgreSQL connected`
- `MongoDB connected`
- `Server listening on 0.0.0.0:3001`

### WebSocket
- `JoinChannel`
- `MessageSend`
- events `Error` avec code (`NOT_MEMBER`, `BANNED`, `INVALID_CHANNEL_ID`, etc.)

## 4) Couverture Et Qualite

### Local
- Backend core coverage:
  - `cd backend && make coverage-core`
- Frontend:
  - `cd frontend/real-time-chat && npm test -- --coverage`
- Desktop:
  - `cd desktop-app && npm test`

### CI
- Le job backend coverage enforce:
  - `--fail-under-lines 70`

## 5) Bugs Potentiels (A Surveiller)

1. Perte de contexte user apres refresh
- Impact: erreurs auth ou mauvais serveur actif.
- Mitigation: synchroniser les stores persistes apres login/logout.

2. Incoherence langue FR/EN
- Impact: labels non traduits sur pages secondaires.
- Mitigation: centraliser les labels UI via helpers i18n tests.

3. Timezone et affichage des dates
- Impact: confusion pour heures et historique messages.
- Mitigation: afficher explicitement timezone ou normaliser UTC.

4. Reconnexion WS en mode degrade
- Impact: UX lente en cas de reseau instable.
- Mitigation: feedback visuel + tentative de refresh token robuste.

5. Scheduler de messages (feature hebdomadaire)
- Impact: message envoye a un horaire inattendu si mauvaise interpretation timezone.
- Mitigation: afficher clairement que la planification utilise UTC.

## 6) Commandes Utiles

- Etat git rapide:
  - `git status --short`
- Jobs Docker:
  - `docker compose ps`
- Logs Postgres:
  - `docker compose logs --tail=80 postgres`
- Verifier Postgres:
  - `docker compose exec -T postgres pg_isready -U epitalk -d epitalk`

## 7) Definition Of Done Debug

Un bug est considere resolu quand:
1. Reproduction initiale documentee.
2. Root cause identifiee.
3. Correctif applique.
4. Tests unitaires/integres passes.
5. Verification manuelle de non-regression effectuee.

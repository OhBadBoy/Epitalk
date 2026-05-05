# EpiTalk - Presentation Stack Technique

## Pitch oral - 2 minutes

EpiTalk est une plateforme de chat temps reel inspiree de Discord, avec des serveurs, des canaux, des messages directs, des reactions et la presence des utilisateurs.

Cote backend, on a choisi Rust avec le framework Axum. Rust nous donne des performances proches du natif, sans garbage collector, et avec une securite memoire garantie a la compilation. Pour une application de chat, c'est important : on doit garder des connexions WebSocket ouvertes longtemps, avec une latence stable.

Axum nous sert a exposer l'API REST et l'endpoint WebSocket. REST gere les operations structurelles comme l'authentification, les serveurs, les canaux et les membres. Le WebSocket gere tout ce qui doit etre instantane : nouveaux messages, typing, presence, reactions, messages directs et heartbeat.

Pour la persistance, le projet utilise deux bases complementaires. PostgreSQL stocke les donnees relationnelles fortes : utilisateurs, serveurs, channels, memberships, invites et bans. MongoDB stocke les messages, les DMs, les reactions, les pins et l'historique, car ce sont des donnees plus flexibles et tres frequemment ecrites.

Cote frontend, on utilise Next.js 16 avec React 19 et TypeScript. Next.js structure l'application, React porte l'interface, TypeScript fiabilise le code. Zustand gere l'etat global avec des stores specialises, et Zod valide les reponses API et les evenements WebSocket avant qu'ils entrent dans l'application.

Pour la securite, les mots de passe sont haches avec Argon2id, recommande par l'OWASP, et l'authentification repose sur des JWT. Pour l'infrastructure, Docker Compose lance localement le backend, PostgreSQL et MongoDB. GitHub Actions assure la qualite avec formatage, lint, build, tests et publication de l'image Docker vers GHCR.

En resume, la stack est coherente avec le besoin produit : un chat temps reel robuste, performant, typé, testable et pret a evoluer vers une architecture plus scalable.

## Plan global

Plan global :
1. Vision du projet
2. Architecture globale
3. Backend
4. Temps reel WebSocket
5. Base de donnees
6. Frontend
7. Desktop Electron
8. Infrastructure / CI-CD
9. Tests / qualite
10. Limites et ameliorations
11. Conclusion

## Mini-checklist avant soutenance

- Lancer une fois le pitch au chronometre : viser 1 min 50 a 2 min 10.
- Preparer une reponse courte sur Rust vs Node/Go.
- Verifier le vocabulaire : Argon2id, JWT HS256, WebSocket Hub, SQLx, Zod.
- Assumer clairement les limites : prod non finalisee, migrations, scaling WebSocket, lockfiles.
- Garder les slides courtes : les details sont dans les notes orales.
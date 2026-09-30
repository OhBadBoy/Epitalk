# EpiTalk — répartition des rôles, branches, fonctionnalités et UML

Ce document définit le périmètre de responsabilité de **James, Daouda, Moïse et Hadrian** dans le découpage officiel du projet d'origine. Il réunit la liste des 14 branches, leurs fonctionnalités et achievements, ainsi que la correspondance des éléments UML frontend, backend et bases de données.

Le dépôt public est une copie du code, sans l'historique complet des contributions de l'équipe. Les 14 noms ci-dessous constituent le **référentiel des branches du projet d'origine** ; ils ne sont pas un inventaire des références Git actuellement publiées dans cette copie. Cette liste ne demande ni la suppression d'autres références existantes ni la création de branches vides pour reconstituer un historique absent.

Une attribution désigne la responsabilité d'un lot de travail. Elle ne prouve ni l'auteur exclusif de chaque ligne, ni la réalisation de toutes ses fonctionnalités, ni la validation d'un achievement. L'intégration peut mobiliser plusieurs participants. L'objectif de couverture de 100 % des achievements, les seuils d'extras et la couverture de tests doivent être confirmés par des preuves distinctes.

## Règles du découpage

- Format obligatoire : `feat/<scope>-<slug>/<owner>`.
- Valeurs autorisées pour `owner` : `hadrian`, `moise`, `daouda`, `james`, sans accents. Le prénom affiché reste **Moïse**.
- **14 branches exactement** dans ce référentiel : aucun lot ajouté ou retiré.
- Chaque branche regroupe un ensemble cohérent d'actions produit.
- Objectif final : couvrir **100 % des achievements** grâce aux lots et à leur intégration. Les mentions « support », « base » et « preuve specs » précisent le type de contribution attendu.
- Le socle RBAC du découpage est **Owner / Admin / Member**. Le rôle **Moderator**, présent dans le code et certaines documentations d'une évolution ultérieure, ne modifie pas rétroactivement le périmètre ni les propriétaires de ces 14 lots.

## Rôles des participants

| Participant | Nombre de branches | Responsabilité principale |
|---|---:|---|
| **James** | **5** | Socle backend, authentification et RBAC ; PostgreSQL ; API REST d'organisation ; CI, couverture et style ; spécifications REST/WS et documentation d'architecture. |
| **Daouda** | **3** | Repository MongoDB des messages ; hub WebSocket et orchestration temps réel ; fonctionnalités supplémentaires. |
| **Moïse** | **3** | Interface serveurs/salons et socle frontend ; interface membres/rôles ; présentation et démonstration. |
| **Hadrian** | **3** | Interface chat et client WebSocket ; tests E2E des achievements ; initialisation des index MongoDB. |
| **Total** | **14** | Coordination et intégration entre les quatre participants. |

## Les 14 branches officielles

### 1. `feat/backend-auth-rbac/james`

**Owner : James.** Lot : Auth + Sessions + RBAC core.

Fonctionnalités :

- Register : inscription.
- Login : connexion.
- Logout : déconnexion.
- `GET /me` : profil courant.
- RBAC Owner/Admin/Member : moteur de règles commun, réutilisé dans les autres fonctionnalités.

**Achievements :** `specs_server`, `specs_client` (support), `user_management` (base).

### 2. `feat/db-postgres-schema-migrations/james`

**Owner : James.** Lot : PostgreSQL schema + migrations, source de vérité pour l'organisation.

Fonctionnalités :

- Persistance de Users, Servers, Channels, Memberships, Invites et Roles.
- Contraintes d'intégrité : clés étrangères (FK) et unicité.

**Achievement :** `persistency`.

Le concept Roles fait partie du modèle de persistance ; cette attribution n'impose pas l'existence d'une table `roles` séparée du rôle porté par les memberships.

### 3. `feat/backend-servers-channels-members/james`

**Owner : James.** Lot : REST API Servers + Channels + Members + Permissions.

Fonctionnalités :

- Servers : list/create/update/delete.
- Rejoindre un serveur via un code d'invitation.
- Quitter un serveur, avec la règle « un owner ne peut pas quitter son serveur ».
- Multi-serveurs : support API pour lister les serveurs et basculer entre eux.
- Members : liste des membres ayant rejoint le serveur.
- Roles : mise à jour du rôle par l'owner ; transfert de propriété **si inclus**.
- Channels : list/create/update/delete, avec permissions admin+ pour les opérations de gestion.

**Achievements :** `server_create`, `server_delete`, `server_join`, `server_multiple`, `server_quit`, `chan_list`, `chan_create`, `chan_delete`, `user_list`, `user_management`.

### 4. `feat/ci-coverage-style/james`

**Owner : James.** Lot : CI + couverture + règles de style backend et frontend.

Fonctionnalités :

- Pipeline lint/test.
- Couverture de tests : **objectif ≥ 70 %**, à mesurer et non considéré comme atteint par cette attribution.
- Conventions de style : rustfmt/Clippy et ESLint/Prettier.

**Achievements :** `versioning_basics` (support), `coding_style`, `tests_unit`, `tests_automation` (support), `tests_coverage`.

### 5. `feat/docs-api-ws-specs/james`

**Owner : James.** Lot : spécifications REST + WS et documentation newcomer.

Fonctionnalités :

- OpenAPI : contrats REST.
- Spécification des événements, payloads et erreurs WebSocket.
- Documentation d'architecture : séparation PostgreSQL / MongoDB.

**Achievements :** `documentation`, `specs_server` / `specs_client` (preuve par les spécifications), `functional-delivery` (support).

### 6. `feat/backend-mongo-message-repo/daouda`

**Owner : Daouda.** Lot : MongoDB Messages repository, historique et persistance des messages.

Fonctionnalités :

- Stockage d'un message dans MongoDB.
- Lecture de l'historique par channel, avec curseur/pagination.
- Suppression d'un message : soft delete **recommandé**.

**Achievements :** `persistency` (messages), `chan_message` (support historique).

### 7. `feat/backend-ws-hub-messages/daouda`

**Owner : Daouda.** Lot : hub WebSocket (`/ws`) + broadcast + présence + typing.

Fonctionnalités :

- Connexion WebSocket et rooms par serveur/channel.
- Envoi d'un message dans un channel via WebSocket et diffusion broadcast.
- Présence online/offline par serveur.
- Typing started/stopped par channel.
- Gestion des connexions multiples, notamment plusieurs onglets.

**Achievements :** `chan_message`, `status_online`, `status_typing`, `specs_server` (temps réel).

### 8. `feat/extra-features/daouda`

**Owner : Daouda.** Lot : extras, hors de la liste de fonctionnalités de base.

Exemples de fonctionnalités du lot, à sélectionner et valider :

- Modifier son propre message.
- Réactions/emoji.
- Mentions `@user`.
- Kick/Ban.
- Profil utilisateur : avatar/statut.

**Achievements :** `extra_small` (≥ 1 extra), `extra_medium` (≥ 3 extras), `extra_large` (> 4 extras).

Ces exemples ne valent pas engagement que les cinq fonctionnalités sont réalisées ; les seuils doivent être justifiés par les extras effectivement livrés.

### 9. `feat/frontend-servers-channels-ui/moise`

**Owner : Moïse.** Lot : UI Serveurs + UI Channels + parcours utilisateurs.

Fonctionnalités :

- UI list/create/delete server.
- UI pour rejoindre un serveur via un code d'invitation.
- UI pour quitter un serveur et basculer entre plusieurs serveurs.
- UI list/create/delete channels selon les permissions.

Le mapping UML ci-dessous lui rattache aussi le shell, l'auth UI, le client REST commun et les stores d'authentification et d'organisation.

**Achievements :** `ui_servers`, `server_create`, `server_delete`, `server_join`, `server_multiple`, `server_quit`, `chan_list`, `chan_create`, `chan_delete`, `specs_client`.

### 10. `feat/frontend-members-roles-ui/moise`

**Owner : Moïse.** Lot : UI Members + Roles, paramètres du serveur.

Fonctionnalités :

- UI de la liste des membres ayant rejoint le serveur.
- Affichage des rôles Owner/Admin/Member.
- Écran de gestion des rôles, accessible à l'owner/admin **selon les règles retenues** ; les autorisations effectives doivent suivre les règles backend.

**Achievements :** `user_list`, `user_management`, `uiux_quality`.

### 11. `feat/docs-presentation-demo/moise`

**Owner : Moïse.** Lot : présentation professionnelle + script de démonstration.

Fonctionnalités :

- Slides ou autre support de présentation, avec plan de démonstration.
- Parcours de démonstration couvrant les achievements clés.

**Achievement :** `presentation`.

### 12. `feat/frontend-chat-realtime-ui/hadrian`

**Owner : Hadrian.** Lot : Chat UI + client WS + UX temps réel.

Fonctionnalités :

- UI du chat : liste des messages, composer et états.
- Affichage des badges online et de l'indicateur de frappe.
- Connexion WebSocket côté frontend et handlers d'événements.
- Historique des messages via REST, avec pagination/curseur.

**Achievements :** `ui_chat`, `ui_design`, `uiux_quality`, `chan_message`, `status_online`, `status_typing`.

### 13. `feat/tests-e2e-achievements/hadrian`

**Owner : Hadrian.** Lot : tests E2E des scénarios achievements.

Fonctionnalités :

- Scénarios : authentification → création/rejoindre un serveur → création d'un channel → envoi d'un message WS → typing → online.
- Tests frontend E2E avec Playwright, conformément au mapping UML.

**Achievements :** `tests_automation`, `tests_coverage` (support), `functional-delivery` (support).

### 14. `feat/db-mongo-indexes/hadrian`

**Owner : Hadrian.** Lot : initialisation MongoDB et index pour les messages/l'historique.

Fonctionnalités :

- Index `{ channel_id: 1, created_at: -1 }`.
- Éventuellement l'index `{ server_id: 1, created_at: -1 }` ; d'autres index par `server_id`/`author_id` restent optionnels dans le mapping UML.
- Convention des identifiants : UUID stockés sous forme de strings.

**Achievement :** `persistency` (support MongoDB).

## Correspondance UML → branches

Les noms et chemins de cette section sont les **éléments logiques du découpage fourni** : ils décrivent les responsabilités, sans garantir une correspondance exacte avec les fichiers actuels. Par exemple, le code peut utiliser `src/repositories/` plutôt que `src/repos/` ou `lib/` plutôt que `src/lib/`. Les sources et rendus [UML historiques](uml/README.md) comportent aussi des noms d'une version antérieure et des éléments `planned` ; ils ne sont pas réécrits par ce référentiel.

### FRONTEND — NextJS

#### Pages (`app/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `ShellLayout (app/(app)/layout.tsx)` | `feat/frontend-servers-channels-ui/moise` | Shell et navigation serveurs/channels. |
| `ShellLayout` : intégration de la zone chat et des états temps réel | `feat/frontend-chat-realtime-ui/hadrian` | Intégration du chat dans le shell commun. |
| `LoginPage (app/(auth)/login)` et `RegisterPage (app/(auth)/register)` | `feat/frontend-servers-channels-ui/moise` | Auth UI de base et redirections. |
| `ServersPage (app/(app)/servers)` et `ServerPage (app/(app)/servers/{serverId})` | `feat/frontend-servers-channels-ui/moise` | Pages serveurs. |
| `ChannelPage (app/(app)/servers/{serverId}/channels/{channelId})` | `feat/frontend-chat-realtime-ui/hadrian` | Page du channel et du chat. |

#### Composants UI (`src/components/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `ServersSidebar`, `ChannelsSidebar`, `Dialogs (Create/Join/Delete/Leave)` | `feat/frontend-servers-channels-ui/moise` | Navigation et opérations serveurs/channels. |
| `MembersPanel`, affichage des rôles Owner/Admin/Member | `feat/frontend-members-roles-ui/moise` | Membres et rôles. |
| `ChatMessageList`, `ChatComposer`, `TypingIndicator`, `OnlineBadges` dans le chat | `feat/frontend-chat-realtime-ui/hadrian` | Composants de messagerie et états temps réel. |

#### Client REST (`src/lib/api/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `FetchClient` et `ZodSchemas` | `feat/frontend-servers-channels-ui/moise` | Socle frontend commun : wrapper fetch, gestion d'erreurs et validation des DTO. |
| `AuthApi` | `feat/frontend-servers-channels-ui/moise` | Appels d'authentification. |
| `ServersApi`, `ChannelsApi` | `feat/frontend-servers-channels-ui/moise` | Appels serveurs/channels. |
| `MessagesApi (history)` | `feat/frontend-chat-realtime-ui/hadrian` | Chargement de l'historique. |

#### Client WebSocket (`src/lib/ws/`)

| Élément UML | Branche responsable |
|---|---|
| `WSClient`, `EventRouter`, `Reconnect/Backoff` | `feat/frontend-chat-realtime-ui/hadrian` |

#### Stores (`src/store/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `AuthStore`, `ServersStore`, `ChannelsStore` | `feat/frontend-servers-channels-ui/moise` | Authentification et organisation. |
| `PresenceStore` (online), `TypingStore` (typing), `ChatStore` (messages) | `feat/frontend-chat-realtime-ui/hadrian` | Production et gestion des états temps réel. |
| Consommation des stores temps réel par `MembersPanel` | `feat/frontend-members-roles-ui/moise` | Consommation de ces états pour le panneau membres ; coordination avec Hadrian. |

#### Tests (`frontend/tests/`)

| Élément UML | Branche responsable |
|---|---|
| Tests E2E Playwright | `feat/tests-e2e-achievements/hadrian` |

### BACKEND — Rust + Axum

#### App et infrastructure

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `Router (Axum)`, `AppState`, `Layers (CORS/Trace/RateLimit)` : mise en place et branchement REST/WS | `feat/backend-auth-rbac/james` | Responsabilité principale : socle et middleware d'authentification. |
| Complément des layers et upgrade WebSocket, si nécessaire | `feat/backend-ws-hub-messages/daouda` | Intégration de la partie WS dans l'infrastructure commune. |
| `config`, `error` (erreurs uniformes), `telemetry` | `feat/backend-auth-rbac/james` | Base commune. |

#### Middlewares

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| Middleware auth : extract user/token/guard | `feat/backend-auth-rbac/james` | Authentification et protections communes. |
| Middleware rate limit anti-spam WS | `feat/backend-ws-hub-messages/daouda` | Limitation côté WebSocket. |
| Middleware rate limit REST global, éventuellement | `feat/backend-auth-rbac/james` | Complément REST selon le besoin. |

#### Routes REST (`src/routes/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `auth.rs` | `feat/backend-auth-rbac/james` | Authentification. |
| `servers.rs`, `channels.rs`, endpoints membres/rôles | `feat/backend-servers-channels-members/james` | Organisation et permissions. |
| `messages.rs`, si l'historique est aussi exposé par REST | `feat/backend-mongo-message-repo/daouda` | Service/repository pour l'historique. |
| Branchement de la route `messages.rs`, si nécessaire | `feat/backend-ws-hub-messages/daouda` ou `feat/backend-servers-channels-members/james` | Option de découpage à coordonner ; pas de propriétaire supplémentaire imposé. |

#### WebSocket (`src/ws/`)

| Élément UML | Branche responsable |
|---|---|
| `ws_upgrade (/ws)`, `protocol` (événements typés), `hub` (rooms/presence/typing) | `feat/backend-ws-hub-messages/daouda` |

#### Services (`src/services/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `auth_service`, `RBAC Policy` Owner/Admin/Member | `feat/backend-auth-rbac/james` | Moteur commun d'authentification et d'autorisation. |
| `server_service`, `channel_service`, permissions incluses | `feat/backend-servers-channels-members/james` | Services d'organisation. |
| `message_service` (write/read/delete), `presence_service` (online/typing) | `feat/backend-ws-hub-messages/daouda` | Orchestration et logique temps réel. |
| Repository MongoDB utilisé par `message_service` | `feat/backend-mongo-message-repo/daouda` | Persistance et accès aux messages. |

#### Repositories (`src/repos/`)

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `postgres/*` : `user_repo`, `server_repo`, `channel_repo`, `membership_repo`, `invite_repo` | `feat/backend-servers-channels-members/james` | Repositories et usages backend. |
| Schéma et migrations utilisés par ces repositories PostgreSQL | `feat/db-postgres-schema-migrations/james` | Structure et intégrité des données. |
| `mongo/message_repo` | `feat/backend-mongo-message-repo/daouda` | Repository des messages. |

#### Tests backend (`backend/tests/`)

| Élément UML | Branche responsable |
|---|---|
| `integration_auth` | `feat/backend-auth-rbac/james` |
| `integration_servers`, `integration_channels` | `feat/backend-servers-channels-members/james` |
| `integration_ws` | `feat/backend-ws-hub-messages/daouda` |

### DB — PostgreSQL + MongoDB

#### PostgreSQL : tables et contraintes

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| `users`, `servers`, `memberships`, `channels`, `invites` | `feat/db-postgres-schema-migrations/james` | Schéma, migrations et contraintes. |
| Utilisation des tables PostgreSQL côté backend | `feat/backend-servers-channels-members/james` | Repositories et services métier. |

#### MongoDB : collection et index

| Élément UML | Branche responsable | Répartition |
|---|---|---|
| Collection `messages` : champs et conventions UUID string | `feat/backend-mongo-message-repo/daouda` | Structure côté code/repository. |
| Index `{ channel_id: 1, created_at: -1 }` ; index optionnels `server_id`/`author_id` | `feat/db-mongo-indexes/hadrian` | Initialisation des index et coordination des conventions UUID string. |

## Coordination et décisions à préserver

| Interface entre lots | Coordination attendue |
|---|---|
| Shell / chat | Moïse maintient le shell et sa navigation ; Hadrian intègre la zone chat et ses états temps réel. |
| Membres / présence / typing | Hadrian fournit les stores ; Moïse les consomme dans `MembersPanel`. |
| API et RBAC / UI | James définit les contrats et règles backend ; Moïse et Hadrian les appliquent dans leurs interfaces. La mention owner/admin « selon règles » dans l'UI n'accorde pas de permissions supplémentaires : la mise à jour des rôles est attribuée à l'owner dans le lot REST. |
| Hub WS / client WS | Daouda et Hadrian coordonnent événements, payloads, erreurs, reconnexion, présence et typing ; James porte la spécification commune. |
| Messages / historique REST | Daouda porte le repository et l'orchestration ; Hadrian consomme l'historique REST. Le mapping conditionne `messages.rs` et laisse le branchement à Daouda ou James : le point d'exposition et son responsable sont à préciser sans ajouter un quinzième lot. |
| MongoDB / index | Daouda définit la structure des messages et les accès ; Hadrian prépare les index. Les identifiants UUID string doivent rester cohérents. |
| PostgreSQL / REST | Les migrations et repositories appartiennent à deux lots de James ; leurs contrats doivent rester compatibles. |
| CI / E2E / démonstration | James porte le pipeline et la mesure de couverture ; Hadrian les scénarios E2E ; Moïse le parcours de démonstration et ses preuves. |

Les choix conditionnels restent conditionnels : transfert de propriété **si inclus**, soft delete **recommandé**, index supplémentaires **optionnels**, rate limit REST **éventuel**, exemples d'extras **à sélectionner**. Le présent document ne tranche pas rétroactivement ces options et ne transforme pas une intention en livraison confirmée.

## Lecture des sources historiques et de l'état actuel

- Ce référentiel fait foi pour **l'attribution des 14 lots fournie par l'équipe**.
- Le [README du dépôt](../README.md) décrit le contexte de la copie publique et son fonctionnement ; le code constitue la source pour vérifier les comportements actuellement implémentés. La répartition prévue ne constitue pas à elle seule une preuve de contribution individuelle effective.
- Les [diagrammes UML](uml/README.md) sont des documents historiques, avec des noms de composants et des éléments `planned` conservés. Un label `planned` ne permet pas de conclure que la fonctionnalité manque encore dans le code.
- Les archives RTC ultérieures à ce découpage, notamment l'[audit technique du 20 mars 2026](audit/RTC_TECH_AUDIT_EXPORT_2026-03-20.md) et la [matrice de remédiation refresh/WS du 20 mars 2026](audit/RTC_REFRESH_WS_REMEDIATION_MATRIX_2026-03-20.md), restent des traces historiques de leur propre périmètre. Elles ne redéfinissent pas automatiquement ces 14 branches ni leurs propriétaires, et leurs constats ne valent pas vérification actuelle.
- Le rôle Moderator et d'autres évolutions visibles dans le code ou les archives sont à lire dans leur contexte ultérieur. Le socle demandé ici demeure Owner/Admin/Member.
- La validation des achievements exige des scénarios exécutés, résultats de tests, mesures et preuves de démonstration ; aucune réussite ou couverture mesurée n'est déclarée par cette matrice de responsabilités.


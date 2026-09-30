# Répartition officielle des 14 branches EpiTalk

La référence complète est [Rôles, branches et correspondances UML](../docs/REPARTITION_ROLES_BRANCHES_UML.md). Elle définit les fonctionnalités, les achievements visés et les coordinations entre participants.

## Règles

- Nommage obligatoire : `feat/<scope>-<slug>/<owner>`.
- `owner` appartient à `hadrian`, `moise`, `daouda`, `james`, sans accents.
- Le référentiel contient exactement **14 branches fonctionnelles**, sans ajout ni retrait.
- Une branche regroupe un ensemble cohérent de fonctionnalités. L'objectif collectif est de couvrir 100 % des achievements grâce à ces lots et à leur intégration.
- Ces noms représentent la répartition du projet d'origine, pas les branches actuellement publiées dans cette copie. Les archives RTC décrivent une autre étape du projet.

## Branches et responsables

| Branche | Owner | Périmètre |
|---|---|---|
| `feat/backend-auth-rbac/james` | James | Authentification, sessions, RBAC et socle backend |
| `feat/db-postgres-schema-migrations/james` | James | Schéma PostgreSQL, migrations et contraintes |
| `feat/backend-servers-channels-members/james` | James | API REST serveurs, salons, membres et permissions |
| `feat/ci-coverage-style/james` | James | CI, couverture et conventions de style |
| `feat/docs-api-ws-specs/james` | James | Contrats REST/WS, architecture et documentation d'accueil |
| `feat/backend-mongo-message-repo/daouda` | Daouda | Repository MongoDB, historique et persistance des messages |
| `feat/backend-ws-hub-messages/daouda` | Daouda | Hub WebSocket, messages, présence et frappe |
| `feat/extra-features/daouda` | Daouda | Fonctionnalités supplémentaires |
| `feat/frontend-servers-channels-ui/moise` | Moïse | UI serveurs/salons, authentification et socle frontend |
| `feat/frontend-members-roles-ui/moise` | Moïse | Interface membres et rôles |
| `feat/docs-presentation-demo/moise` | Moïse | Présentation et scénario de démonstration |
| `feat/frontend-chat-realtime-ui/hadrian` | Hadrian | Interface chat, client WS, historique et états temps réel |
| `feat/tests-e2e-achievements/hadrian` | Hadrian | Scénarios E2E des achievements |
| `feat/db-mongo-indexes/hadrian` | Hadrian | Initialisation des index MongoDB et convention des identifiants |

**Répartition : James 5, Daouda 3, Moïse 3, Hadrian 3.** L'attribution désigne le responsable du lot ; elle ne constitue pas une preuve d'implémentation ou de validation des achievements.

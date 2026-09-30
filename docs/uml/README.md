# Diagrammes UML EpiTalk

Le [référentiel des rôles, des 14 branches et de leur correspondance UML](../REPARTITION_ROLES_BRANCHES_UML.md) définit la répartition entre James, Daouda, Moïse et Hadrian pour les trois couches du projet.

## Sources et rendus existants

| Couche | Source PlantUML | Rendu PNG |
|---|---|---|
| Frontend NextJS | [EPITALK_FRONTEND.puml](EPITALK_FRONTEND.puml) | [EPITALK_FRONTEND.png](EPITALK_FRONTEND.png) |
| Backend Rust + Axum | [EPITALK_BACKEND.puml](EPITALK_BACKEND.puml) | [EPITALK_BACKEND.png](EPITALK_BACKEND.png) |
| Bases PostgreSQL + MongoDB | [EPITALK_DB.puml](EPITALK_DB.puml) | [EPITALK_DB.png](EPITALK_DB.png) |

## Statut et lecture

Ces sources et rendus sont des **documents historiques conservés**. Ils comportent des noms de composants ou chemins d'une version antérieure, ainsi que des éléments `planned` / `[PLANNED]`. Ils ne constituent pas une cartographie vérifiée de l'ensemble des fichiers actuels ; un élément marqué `planned` ne prouve pas qu'il reste à implémenter aujourd'hui.

La correspondance UML → branche est détaillée dans le [référentiel](../REPARTITION_ROLES_BRANCHES_UML.md). Ses chemins sont logiques : des emplacements tels que `src/repos/`, `src/lib/` ou `frontend/tests/` peuvent différer des chemins actuels. Cette documentation des responsabilités ne réécrit pas les trois sources `.puml` ni leurs rendus `.png`.

Le socle RBAC du découpage demandé est **Owner / Admin / Member**. Le rôle **Moderator** apparaît notamment dans le diagramme DB et le code d'une évolution ultérieure ; sa présence ne change pas les propriétaires ni le périmètre des 14 lots.

Le dépôt public est une copie sans l'historique complet de l'équipe. Les 14 branches du référentiel désignent le découpage du projet d'origine, et non un inventaire des références Git publiées ici. Les archives RTC plus récentes, dont l'[audit technique](../audit/RTC_TECH_AUDIT_EXPORT_2026-03-20.md) et la [matrice de remédiation refresh/WS](../audit/RTC_REFRESH_WS_REMEDIATION_MATRIX_2026-03-20.md), restent des documents historiques distincts.

Consulter le [README du dépôt](../../README.md) et le code pour l'état actuel du projet. Une responsabilité attribuée, un diagramme ou un objectif d'achievement ne suffit pas à établir l'auteur exclusif d'une implémentation ni sa validation effective.

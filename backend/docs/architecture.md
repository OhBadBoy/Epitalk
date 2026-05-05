# Architecture

High-level architecture:

- Axum HTTP server exposing REST endpoints and WebSocket upgrade
- WebSocket hub (`src/ws/*`) for routing messages and rooms
- Messages stored in MongoDB (collection `messages`)
- Users planned to be stored in Postgres (schema in `scripts/init-postgres.sql`)

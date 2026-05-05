# RTC Refresh/WS Remediation Matrix (2026-03-20)

## Scope
- Backend auth refresh behavior
- Frontend session restoration on reload
- WebSocket reconnect/logout consistency
- 401 handling consistency in active user flows

## Root Causes Identified
1. Refresh endpoint rejected expired tokens because it used strict auth extractor.
2. WebSocket reconnect could use stale token snapshots during delayed retries.
3. Session invalidation paths were duplicated and inconsistent across UI components.
4. Some 401 UI paths only displayed local errors instead of terminating session + redirecting.
5. Hydration timing could temporarily desync in-memory token and persisted token usage.

## Remediations Applied
| ID | Area | File(s) | Change | Validation |
|---|---|---|---|---|
| R1 | Backend refresh | backend/src/auth/jwt.rs | Added validation mode allowing expired tokens while still verifying signature (`validate_token_allow_expired`). | `cargo test` |
| R2 | Backend refresh | backend/src/auth/middleware.rs | Added `RequireAuthAllowExpired` extractor for refresh-only auth flow. | `cargo test` |
| R3 | Backend refresh | backend/src/routes/auth.rs | Switched `/api/auth/refresh` to `RequireAuthAllowExpired`. | `cargo test` |
| R4 | Backend tests | backend/src/auth/jwt.rs | Added regression tests: allow expired signed token; reject tampered token. | `cargo test` |
| R5 | WS handshake | backend/src/ws/ws_upgrade.rs | Standardized invalid WS token response to HTTP 401 JSON payload. | build/tests pass |
| R6 | WS reconnect | frontend/real-time-chat/store/websocket.store.ts | Added bounded reconnect attempts, token freshness check, session check before reconnect. | `npm run build` |
| R7 | Auth hydration | frontend/real-time-chat/store/auth.store.ts | Added `hasHydrated` flag via persist rehydration callback. | `npm run build` |
| R8 | Route guards | frontend/real-time-chat/app/(app)/layout.tsx; frontend/real-time-chat/app/(auth)/layout.tsx | Added hydration-aware session checks and redirects. | `npm run build` |
| R9 | API refresh | frontend/real-time-chat/lib/api/fetchClient.ts | Added 401 refresh+retry flow and single-flight refresh lock. | `npm run build` |
| R10 | 401 handling | frontend/real-time-chat/app/invite/[code]/invite-join-client.tsx; frontend/real-time-chat/app/(app)/servers/components/channels-sidebar.tsx; frontend/real-time-chat/app/(app)/servers/components/members-panel.tsx; frontend/real-time-chat/app/(app)/servers/components/servers-loader.tsx | Unified 401 behavior to terminate session and redirect to login. | `npm run build` |
| R11 | Session source of truth | frontend/real-time-chat/lib/auth/session.ts + call sites | Centralized session termination (`terminateSession`) to remove duplicated disconnect/logout logic. | `npm run build` |
| R12 | Auth cleanup consistency | frontend/real-time-chat/lib/api/auth.api.ts | `logout()` now clears both `token` and `auth-storage`. | `npm run build` |

## Verification Summary
- Backend: `cargo test` passed (61 tests).
- Frontend: `npm run build` passed after each remediation increment.

## Residual Risks
1. Frontend has no dedicated automated integration tests for browser refresh flows.
2. Recovery UX messaging remains component-local (not globally coordinated).
3. Realtime reconnection policy is conservative but not yet load-tested under unstable networks.

## Recommended Next Steps
1. Add frontend integration tests for: valid refresh, expired token refresh, corrupted token, invite flow 401.
2. Add a global auth boundary/toast for session-expired events.
3. Add WS reconnect telemetry counters (attempts, failures, terminal disconnect reason).

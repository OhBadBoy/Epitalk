# RTC Technical Audit Export (2026-03-20)

## Scope
Audit performed against:
- PDF requirements: RTC_strikes_back-project.pdf
- Branches requested:
  - feat/backend-web-server/james
  - feat/backend-core-kick/james
  - feat/backend-core-ban-perm/james
  - feat/extra-message-search/james
  - feat/extra-pinned-messages/james

## Executive Summary
- Only one audited branch contains unique and attributable delivery work: feat/backend-core-ban-perm/james.
- Four requested branches have no unique commits vs main and cannot be treated as feature-deliverable branches.
- Phase 1 from the PDF is only partially covered in this branch set.
- Extras message search and pinned messages are not implemented in their dedicated branches.

## PDF Requirement Extraction (Phase-Focused)
### Phase 1 (Core Features)
- Kick member from server (can rejoin): REQUIRED
- Permanent ban: REQUIRED
- Temporary ban with expiry: REQUIRED
- Message editing (author edits own message): REQUIRED

### Phase 2 (Enhancement / Professionalization)
- Frontend i18n FR/EN
- CI/CD on branch push and merge to main
- Auto build on tag
- GIF API integration
- Private messages
- Reactions

### Phase 3 (Desktop)
- Installable desktop app (Windows/macOS/Linux)
- Full GUI for all features
- FR/EN multilingual
- System notifications
- Tech: Tauri or Electron

## Branch-by-Branch Findings
### 1) feat/backend-web-server/james
- Unique commits vs main: 0
- Status: STALE for branch-level delivery traceability
- Functional observations in shared code state:
  - REST + WebSocket server exists
  - WS JWT auth exists
  - Multi-connection hub exists
- Conclusion:
  - Feature foundation exists in codebase, but not attributable as unique branch delivery.

### 2) feat/backend-core-kick/james
- Analyzed as: origin/feat/backend-core-kick/james
- Unique commits vs main: 0
- Status: STALE for branch-level delivery traceability
- Functional observations in shared code state:
  - Kick endpoint exists in members routes
  - Membership deletion logic exists
- Conclusion:
  - Kick behavior appears in current shared state, but this branch does not carry unique commits.

### 3) feat/backend-core-ban-perm/james
- Unique commits vs main: 8
- Status: VALID (real delivery branch)
- Functional coverage:
  - Ban persistence migration with expires_at (permanent when NULL)
  - Ban/unban/list bans endpoints
  - is_banned check integrated in invite join flow
- Quality notes:
  - Integration tests are mostly scaffold/ignored
  - WS real-time enforcement after a ban is not fully enforced in the connection pipeline

### 4) feat/extra-message-search/james
- Analyzed as: origin/feat/extra-message-search/james
- Unique commits vs main: 0
- Status: STALE
- Search implementation evidence:
  - No dedicated search endpoint detected
  - No search service/repository method detected
  - No search UI/API client evidence detected
- Conclusion: NOT IMPLEMENTED in this branch state.

### 5) feat/extra-pinned-messages/james
- Analyzed as: origin/feat/extra-pinned-messages/james
- Unique commits vs main: 0
- Status: STALE
- Pinned implementation evidence:
  - No pin/unpin/list pinned endpoint detected
  - No pin persistence model detected
  - No pin UI/API client evidence detected
- Conclusion: NOT IMPLEMENTED in this branch state.

## Requirement Coverage Matrix (Requested Scope)
- Kick: PARTIAL (present in shared code, non-attributable branch)
- Permanent ban: IMPLEMENTED (backend-core-ban-perm)
- Temporary ban: IMPLEMENTED (backend-core-ban-perm)
- Message editing: MISSING in this audited branch set
- Extra message search: MISSING in branch claimed for it
- Extra pinned messages: MISSING in branch claimed for it

## Critical Risks
### Blocking
- Branch governance/traceability issue:
  - Multiple branches point to the same commit and do not contain unique feature work.
  - Impact: feature ownership, reviewability, and delivery proof are unreliable.

### Important
- Runtime ban consistency gap:
  - Ban is persisted and checked at invite join, but active WS sessions are not clearly terminated or blocked after ban.
  - Impact: banned users may continue to interact in real time until reconnect.

### Important
- Validation gap:
  - Integration tests for sensitive moderation flows are largely scaffold/ignored.
  - Impact: regression risk during demo and merge.

## Recommended Priority Order
1. Fix branch traceability baseline (declare stale branches and align real source branches)
2. Enforce ban behavior in WebSocket runtime (active-session handling)
3. Implement missing Phase 1 item (message editing)
4. Implement extras only after Phase 1 compliance is complete (search, pinned)
5. Harden test coverage for moderation and WS flows

## Single Next Action (as of this export)
- Action: enforce immediate ban effect on active WS sessions and channel actions.
- Why first: highest functional/security gap after branch traceability.

## Evidence Notes
- Unique-commit checks and branch comparisons were performed with git log and merge-base comparisons.
- Feature presence checks were performed by direct inspection of backend routes/services/repositories and targeted grep on branch trees.

---
Export generated automatically on 2026-03-20.

import { mkdirSync, writeFileSync } from "fs";
import { join } from "path";

const outDir = join(process.cwd(), "docs", "uml");
mkdirSync(outDir, { recursive: true });

const FRONTEND = `@startuml
title EpiTalk — FRONTEND (NextJS) — Component UML
skinparam componentStyle rectangle
skinparam shadowing false
skinparam packageStyle rectangle

package "frontend/ (NextJS App Router)" {

  package "Pages (app/)" {
    component "ShellLayout\\napp/(app)/layout.tsx" as ShellLayout
    component "LoginPage\\napp/(auth)/login" as LoginPage
    component "RegisterPage\\napp/(auth)/register" as RegisterPage
    component "ServersPage\\napp/(app)/servers" as ServersPage
    component "ServerPage\\napp/(app)/servers/{serverId}" as ServerPage
    component "ChannelPage\\napp/(app)/servers/{serverId}/channels/{channelId}" as ChannelPage
  }

  package "UI Components (src/components/)" {
    component "ServersSidebar" as ServersSidebar
    component "ChannelsSidebar" as ChannelsSidebar
    component "MembersPanel" as MembersPanel
    component "ChatMessageList" as ChatMessageList
    component "ChatComposer" as ChatComposer
    component "TypingIndicator" as TypingIndicator
    component "OnlineBadges" as OnlineBadges
    component "Dialogs\\n(Create/Join/Delete/Leave)" as Dialogs
  }

  package "REST Client (src/lib/api/)" {
    component "FetchClient" as FetchClient
    component "AuthApi" as AuthApi
    component "ServersApi" as ServersApi
    component "ChannelsApi" as ChannelsApi
    component "MessagesApi\\n(history)" as MessagesApi
  }

  package "WebSocket Client (src/lib/ws/)" {
    component "WSClient" as WSClient
    component "EventRouter" as EventRouter
    component "Reconnect/Backoff" as Reconnect
  }

  package "State (src/store/)" {
    component "AuthStore" as AuthStore
    component "ServersStore" as ServersStore
    component "ChannelsStore" as ChannelsStore
    component "ChatStore" as ChatStore
    component "PresenceStore" as PresenceStore
    component "TypingStore" as TypingStore
  }

  package "Validation (src/lib/validators/)" {
    component "ZodSchemas\\n(DTO validation)" as ZodSchemas
  }

  package "Tests (frontend/tests/)" {
    component "Unit Tests\\n(vitest/jest)" as FEUnit
    component "E2E Tests\\n(Playwright)" as FEE2E
  }
}

cloud "Backend HTTP (REST)" as BE_HTTP
cloud "Backend WS (/ws)" as BE_WS

' UI composition
ShellLayout --> ServersSidebar
ShellLayout --> ChannelsSidebar
ShellLayout --> MembersPanel

ServersPage --> Dialogs
ServerPage  --> Dialogs
ChannelPage --> ChatMessageList
ChannelPage --> ChatComposer
ChannelPage --> TypingIndicator
ServerPage  --> OnlineBadges

' REST flow
LoginPage --> AuthApi
RegisterPage --> AuthApi
ServersPage --> ServersApi
ServerPage  --> ServersApi
ServerPage  --> ChannelsApi
ChannelPage --> MessagesApi

AuthApi --> FetchClient
ServersApi --> FetchClient
ChannelsApi --> FetchClient
MessagesApi --> FetchClient

FetchClient --> ZodSchemas
FetchClient --> BE_HTTP : HTTPS JSON

' WS flow
WSClient --> Reconnect
WSClient --> ZodSchemas
WSClient --> BE_WS : WebSocket events
EventRouter <-- WSClient

' Stores update
AuthApi --> AuthStore
ServersApi --> ServersStore
ChannelsApi --> ChannelsStore
MessagesApi --> ChatStore

EventRouter --> ChatStore : message.created/deleted
EventRouter --> PresenceStore : presence.updated
EventRouter --> TypingStore : typing.started/stopped

' Tests coverage
FEUnit ..> FetchClient
FEUnit ..> WSClient
FEE2E ..> ShellLayout
@enduml
`;

const BACKEND = `@startuml
title EpiTalk — BACKEND (Rust + Axum) — Component UML
skinparam componentStyle rectangle
skinparam shadowing false
skinparam packageStyle rectangle

package "backend/ (Rust + Axum)" {

  package "App (src/app/)" {
    component "Router (Axum)" as Router
    component "AppState" as AppState
    component "Layers\\n(CORS/Trace/RateLimit)" as Layers
  }

  package "Middlewares (src/middlewares/)" {
    component "AuthMiddleware" as AuthMiddleware
    component "RateLimitMiddleware" as RateLimitMiddleware
  }

  package "REST Routes (src/routes/)" {
    component "auth.rs" as AuthRoutes
    component "servers.rs" as ServerRoutes
    component "channels.rs" as ChannelRoutes
    component "messages.rs" as MessageRoutes
  }

  package "WS (src/ws/)" {
    component "ws_upgrade\\n(/ws)" as WsUpgrade
    component "protocol\\n(typed events)" as WsProtocol
    component "hub\\n(rooms/presence/typing)" as WsHub
  }

  package "Services (src/services/)" {
    component "AuthService" as AuthService
    component "ServerService" as ServerService
    component "ChannelService" as ChannelService
    component "MessageService" as MessageService
    component "PresenceService" as PresenceService
  }

  package "Policies (src/policies/)" {
    component "RBAC Policy\\n(Owner/Admin/Member)" as RbacPolicy
  }

  package "Repos (src/repos/)" {
    package "postgres/" {
      component "UserRepo" as UserRepo
      component "ServerRepo" as ServerRepo
      component "ChannelRepo" as ChannelRepo
      component "MembershipRepo" as MembershipRepo
      component "InviteRepo" as InviteRepo
    }
    package "mongo/" {
      component "MessageRepo (Mongo)" as MongoMessageRepo
    }
  }

  package "Infra (config/error/telemetry)" {
    component "Config (env validation)" as Config
    component "Error (uniform API/WS errors)" as Error
    component "Telemetry (logs/trace)" as Telemetry
  }

  package "Tests (backend/tests/)" {
    component "integration_auth" as TAuth
    component "integration_servers" as TServers
    component "integration_channels" as TChannels
    component "integration_ws" as TWs
  }
}

cloud "Frontend (NextJS)" as FE
database "PostgreSQL" as PG
database "MongoDB" as MG

' Entry points
FE --> Router : REST JSON
FE --> WsUpgrade : WebSocket connect

' App wiring
Router --> Layers
Layers --> AuthMiddleware
Layers --> RateLimitMiddleware

Router --> AuthRoutes
Router --> ServerRoutes
Router --> ChannelRoutes
Router --> MessageRoutes

' REST -> Services -> Repos
AuthRoutes --> AuthService
ServerRoutes --> ServerService
ChannelRoutes --> ChannelService
MessageRoutes --> MessageService

AuthService --> RbacPolicy
ServerService --> RbacPolicy
ChannelService --> RbacPolicy
MessageService --> RbacPolicy

AuthService --> UserRepo
ServerService --> ServerRepo
ServerService --> MembershipRepo
ServerService --> InviteRepo
ChannelService --> ChannelRepo
MessageService --> MongoMessageRepo

UserRepo --> PG
ServerRepo --> PG
MembershipRepo --> PG
InviteRepo --> PG
ChannelRepo --> PG

MongoMessageRepo --> MG

' WS wiring
WsUpgrade --> WsProtocol
WsUpgrade --> WsHub
WsHub --> PresenceService
WsHub --> MessageService
PresenceService --> MembershipRepo : validate membership
MessageService --> MongoMessageRepo

' Cross-cutting infra
Router ..> Config
Router ..> Telemetry
AuthRoutes ..> Error
WsHub ..> Error

' Tests
TAuth ..> AuthRoutes
TServers ..> ServerRoutes
TChannels ..> ChannelRoutes
TWs ..> WsUpgrade
@enduml
`;

const DB = `@startuml
title EpiTalk — DB (PostgreSQL + MongoDB) — UML (ER + Collections)
hide circle
skinparam shadowing false
skinparam classAttributeIconSize 0

package "PostgreSQL (organisation / RBAC)" {

  class users <<table>> {
    +id: UUID (PK)
    email: CITEXT (UNIQUE)
    password_hash: TEXT
    created_at: TIMESTAMP
  }

  class servers <<table>> {
    +id: UUID (PK)
    name: TEXT
    owner_id: UUID (FK -> users.id)
    created_at: TIMESTAMP
  }

  class memberships <<table>> {
    +server_id: UUID (PK, FK -> servers.id)
    +user_id: UUID (PK, FK -> users.id)
    role: ENUM('OWNER','ADMIN','MEMBER')
    joined_at: TIMESTAMP
  }

  class channels <<table>> {
    +id: UUID (PK)
    server_id: UUID (FK -> servers.id)
    name: TEXT
    kind: ENUM('TEXT')
    created_at: TIMESTAMP
  }

  class invites <<table>> {
    +code: TEXT (PK)
    server_id: UUID (FK -> servers.id)
    created_by: UUID (FK -> users.id)
    expires_at: TIMESTAMP?
    max_uses: INT?
    uses: INT
    created_at: TIMESTAMP
  }

  users "1" -- "many" servers : owns >
  servers "1" -- "many" channels
  users "1" -- "many" memberships
  servers "1" -- "many" memberships
  servers "1" -- "many" invites
  users "1" -- "many" invites : creates >
}

package "MongoDB (messages / history)" {

  class messages <<collection>> {
    +_id: ObjectId (PK)
    channel_id: UUID (string)
    server_id: UUID (string)
    author_id: UUID (string)
    content: string
    created_at: datetime
    deleted_at: datetime?
  }

  note right of messages
    Indexes (recommended):
      - { channel_id: 1, created_at: -1 }
      - { server_id: 1, created_at: -1 } (optional)
      - { author_id: 1, created_at: -1 } (optional)
  end note
}

' Logical references across DBs (not enforced)
channels ..> messages : channel_id
users ..> messages : author_id
servers ..> messages : server_id
@enduml
`;

writeFileSync(join(outDir, "EPITALK_FRONTEND.puml"), FRONTEND, "utf8");
writeFileSync(join(outDir, "EPITALK_BACKEND.puml"), BACKEND, "utf8");
writeFileSync(join(outDir, "EPITALK_DB.puml"), DB, "utf8");

console.log("✅ Generated PlantUML files in:", outDir);
console.log(" - EPITALK_FRONTEND.puml");
console.log(" - EPITALK_BACKEND.puml");
console.log(" - EPITALK_DB.puml");

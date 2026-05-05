import fs from "node:fs/promises";
import path from "node:path";
import {
  Presentation,
  PresentationFile,
  row,
  column,
  grid,
  layers,
  panel,
  text,
  shape,
  rule,
  fill,
  hug,
  fixed,
  wrap,
  grow,
  fr,
  auto,
} from "@oai/artifact-tool";

const W = 1920;
const H = 1080;

const ROOT = path.resolve(".");
const OUT = path.join(ROOT, "output");
const SCRATCH = path.join(ROOT, "scratch");

const P = {
  bg: "#060A16",
  bg2: "#0B1020",
  surface: "#10182D",
  surface2: "#141E37",
  surface3: "#0D1428",
  line: "#26324D",
  ink: "#F8FAFC",
  muted: "#A7B0C6",
  dim: "#667085",
  blue: "#2F80FF",
  blue2: "#0EA5E9",
  cyan: "#22D3EE",
  violet: "#8B5CF6",
  green: "#34D399",
  amber: "#FBBF24",
  red: "#FB7185",
  white: "#FFFFFF",
};

const FONT = "Aptos Display";
const BODY = "Aptos";

const pitch =
  "EpiTalk est une plateforme de chat temps reel inspiree de Discord, avec des serveurs, des canaux, des messages directs, des reactions et la presence des utilisateurs.\n\n" +
  "Cote backend, on a choisi Rust avec le framework Axum. Rust nous donne des performances proches du natif, sans garbage collector, et avec une securite memoire garantie a la compilation. Pour une application de chat, c'est important : on doit garder des connexions WebSocket ouvertes longtemps, avec une latence stable.\n\n" +
  "Axum nous sert a exposer l'API REST et l'endpoint WebSocket. REST gere les operations structurelles comme l'authentification, les serveurs, les canaux et les membres. Le WebSocket gere tout ce qui doit etre instantane : nouveaux messages, typing, presence, reactions, messages directs et heartbeat.\n\n" +
  "Pour la persistance, le projet utilise deux bases complementaires. PostgreSQL stocke les donnees relationnelles fortes : utilisateurs, serveurs, channels, memberships, invites et bans. MongoDB stocke les messages, les DMs, les reactions, les pins et l'historique, car ce sont des donnees plus flexibles et tres frequemment ecrites.\n\n" +
  "Cote frontend, on utilise Next.js 16 avec React 19 et TypeScript. Next.js structure l'application, React porte l'interface, TypeScript fiabilise le code. Zustand gere l'etat global avec des stores specialises, et Zod valide les reponses API et les evenements WebSocket avant qu'ils entrent dans l'application.\n\n" +
  "Pour la securite, les mots de passe sont haches avec Argon2id, recommande par l'OWASP, et l'authentification repose sur des JWT. Pour l'infrastructure, Docker Compose lance localement le backend, PostgreSQL et MongoDB. GitHub Actions assure la qualite avec formatage, lint, build, tests et publication de l'image Docker vers GHCR.\n\n" +
  "En resume, la stack est coherente avec le besoin produit : un chat temps reel robuste, performant, typé, testable et pret a evoluer vers une architecture plus scalable.";

const plan =
  "Plan global :\n" +
  "1. Vision du projet\n" +
  "2. Architecture globale\n" +
  "3. Backend\n" +
  "4. Temps reel WebSocket\n" +
  "5. Base de donnees\n" +
  "6. Frontend\n" +
  "7. Desktop Electron\n" +
  "8. Infrastructure / CI-CD\n" +
  "9. Tests / qualite\n" +
  "10. Limites et ameliorations\n" +
  "11. Conclusion";

function note({ objective, visual, oral }) {
  return [
    `Objectif de la slide : ${objective}`,
    `Suggestion visuelle : ${visual}`,
    "",
    "Notes orales :",
    oral,
  ].join("\n");
}

function t(value, options = {}) {
  return text(value, {
    width: options.width ?? fill,
    height: options.height ?? hug,
    name: options.name,
    columnSpan: options.columnSpan,
    rowSpan: options.rowSpan,
    style: {
      fontFace: options.fontFace ?? BODY,
      fontSize: options.size ?? 28,
      color: options.color ?? P.ink,
      bold: options.bold ?? false,
      italic: options.italic ?? false,
      lineSpacing: options.lineSpacing,
      ...(options.style ?? {}),
    },
  });
}

function title(value, subtitle) {
  const children = [
    t(value, {
      name: "slide-title",
      size: 54,
      bold: true,
      fontFace: FONT,
      width: fill,
      color: P.ink,
    }),
  ];
  if (subtitle) {
    children.push(
      t(subtitle, {
        name: "slide-subtitle",
        size: 23,
        width: fill,
        color: P.muted,
      }),
    );
  }
  return column({ name: "title-stack", width: fill, height: hug, gap: 14 }, children);
}

function badge(value, color = P.blue) {
  return panel(
    {
      width: hug,
      height: hug,
      padding: { x: 18, y: 8 },
      borderRadius: "rounded-full",
      fill: `${color}22`,
      line: { color: `${color}88`, width: 1 },
    },
    t(value, { size: 17, bold: true, color, width: hug }),
  );
}

function bullet(value, color = P.blue) {
  return row({ width: fill, height: hug, gap: 14, align: "start" }, [
    shape({
      geometry: "ellipse",
      width: fixed(11),
      height: fixed(11),
      fill: color,
      line: { color, width: 0 },
    }),
    t(value, { size: 25, color: P.ink, width: fill }),
  ]);
}

function smallBullet(value, color = P.cyan) {
  return row({ width: fill, height: hug, gap: 12, align: "start" }, [
    shape({
      geometry: "ellipse",
      width: fixed(8),
      height: fixed(8),
      fill: color,
      line: { color, width: 0 },
    }),
    t(value, { size: 22, color: P.muted, width: fill }),
  ]);
}

function card(label, body, accent = P.blue, options = {}) {
  return panel(
    {
      name: options.name,
      width: options.width ?? fill,
      height: options.height ?? hug,
      padding: options.padding ?? { x: 28, y: 22 },
      borderRadius: "rounded-md",
      fill: options.fill ?? P.surface,
      line: { color: `${accent}66`, width: 1 },
    },
    column({ width: fill, height: hug, gap: 10 }, [
      t(label, { size: options.labelSize ?? 22, bold: true, color: accent }),
      t(body, { size: options.bodySize ?? 24, color: options.bodyColor ?? P.ink, width: fill }),
    ]),
  );
}

function stackLabel(label, accent = P.blue) {
  return panel(
    {
      width: fill,
      height: fixed(64),
      padding: { x: 18, y: 12 },
      borderRadius: "rounded-md",
      fill: `${accent}20`,
      line: { color: `${accent}88`, width: 1 },
      align: "center",
      justify: "center",
    },
    t(label, { size: 23, bold: true, color: P.ink, width: fill }),
  );
}

function addFooter(rootChildren, indexLabel = "") {
  rootChildren.push(
    row({ width: fill, height: hug, align: "center", justify: "between" }, [
      t("EpiTalk - Stack technique", { size: 14, color: P.dim, width: wrap(500) }),
      t(indexLabel, { size: 14, color: P.dim, width: wrap(300) }),
    ]),
  );
}

function composeSlide(pres, bg = P.bg, content, notes) {
  const slide = pres.slides.add();
  slide.background.fill = bg;
  slide.compose(content, {
    frame: { left: 0, top: 0, width: W, height: H },
    baseUnit: 8,
  });
  if (notes) slide.speakerNotes.setText(notes);
  return slide;
}

function createDeck() {
  const presentation = Presentation.create({ slideSize: { width: W, height: H } });

  // 1. Title
  composeSlide(
    presentation,
    P.bg,
    grid(
      {
        name: "cover-root",
        width: fill,
        height: fill,
        columns: [fr(1.05), fr(0.95)],
        padding: { x: 92, y: 82 },
        columnGap: 72,
      },
      [
        column({ width: fill, height: fill, gap: 28, justify: "center" }, [
          row({ width: fill, height: hug, gap: 14 }, [badge("Soutenance technique", P.cyan), badge("Stack projet", P.violet)]),
          t("EpiTalk", {
            size: 118,
            bold: true,
            fontFace: FONT,
            color: P.white,
            width: fill,
          }),
          t("Presentation de la stack technique", {
            size: 42,
            bold: true,
            color: P.blue,
            width: fill,
          }),
          rule({ width: fixed(280), stroke: P.blue, weight: 5 }),
          t("Chat temps reel inspire de Discord - Rust, Axum, Next.js, WebSocket, PostgreSQL, MongoDB", {
            size: 28,
            color: P.muted,
            width: wrap(850),
          }),
        ]),
        panel(
          {
            width: fill,
            height: fill,
            padding: { x: 38, y: 42 },
            borderRadius: "rounded-lg",
            fill: P.surface3,
            line: { color: P.line, width: 1 },
          },
          column({ width: fill, height: fill, gap: 20, justify: "center" }, [
            stackLabel("Frontend - Next.js 16 / React 19", P.blue),
            stackLabel("Backend - Rust / Axum / Tokio", P.violet),
            stackLabel("Temps reel - WebSocket Hub", P.cyan),
            row({ width: fill, height: hug, gap: 16 }, [
              stackLabel("PostgreSQL", P.green),
              stackLabel("MongoDB", P.amber),
            ]),
            stackLabel("Docker + GitHub Actions + GHCR", P.blue2),
          ]),
        ),
      ],
    ),
    note({
      objective: "Installer le sujet et donner une premiere lecture premium de la stack.",
      visual: "Couverture sombre, titre monumental, colonne de technologies sous forme de pipeline.",
      oral:
        "Bonjour, je vais vous presenter la stack technique d'EpiTalk. Le but n'est pas seulement de lister des technologies, mais de montrer pourquoi elles forment une architecture coherente pour un chat temps reel : un frontend moderne, un backend performant, deux bases de donnees avec des responsabilites distinctes, et une couche WebSocket dediee au temps reel.",
    }),
  );

  // 2. Pitch
  composeSlide(
    presentation,
    P.bg2,
    column(
      { width: fill, height: fill, padding: { x: 92, y: 74 }, gap: 42 },
      [
        title("Pitch oral - 2 minutes", "La version complete est dans les notes orales."),
        grid(
          { width: fill, height: fill, columns: [fr(1), fr(1)], rows: [auto, auto], columnGap: 28, rowGap: 28 },
          [
            card("Produit", "EpiTalk : chat temps reel inspire de Discord", P.cyan),
            card("Backend", "Rust / Axum pour performance, securite et WebSocket", P.violet),
            card("Donnees", "PostgreSQL pour le relationnel, MongoDB pour les messages", P.green),
            card("Frontend + Infra", "Next.js, TypeScript, Docker, GitHub Actions", P.blue),
          ],
        ),
        row({ width: fill, height: hug, justify: "between", align: "center" }, [
          badge("A apprendre par coeur", P.amber),
          t("Plan : vision -> architecture -> couches -> qualite -> limites -> conclusion", {
            size: 18,
            color: P.dim,
            width: wrap(950),
          }),
        ]),
      ],
    ),
    [pitch, "", plan].join("\n"),
  );

  // 3. Vision
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 74 }, gap: 38 }, [
      title("Vision du projet", "Une plateforme de communication temps reel, inspiree de Discord."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1.05)], columnGap: 58 }, [
        column({ width: fill, height: fill, gap: 22, justify: "center" }, [
          t("EpiTalk centralise les echanges autour de serveurs, canaux, DMs et presence.", {
            size: 43,
            bold: true,
            fontFace: FONT,
            color: P.ink,
            width: fill,
          }),
          t("La stack est orientee interaction instantanee, robustesse et maintenabilite.", {
            size: 26,
            color: P.muted,
            width: fill,
          }),
        ]),
        column({ width: fill, height: fill, gap: 20, justify: "center" }, [
          bullet("Serveurs, canaux et messages directs", P.blue),
          bullet("Messages, reactions, pins et historique", P.violet),
          bullet("Typing et presence utilisateur en temps reel", P.cyan),
          bullet("Client web + client desktop Electron", P.green),
        ]),
      ]),
      row({ width: fill, height: hug, justify: "between" }, [badge("Objectif : experience chat fluide", P.blue), t("Source : PRESENTATION_STACK_PROJET.md", { size: 14, color: P.dim, width: wrap(430) })]),
    ]),
    note({
      objective: "Expliquer le produit avant de parler technique.",
      visual: "Grand message a gauche, capacites produit a droite.",
      oral:
        "Avant d'entrer dans la stack, il faut comprendre le besoin. EpiTalk est une plateforme de chat temps reel inspiree de Discord. On retrouve les notions de serveurs, de canaux, de messages directs, de reactions, de pins et de presence. Ce type de produit impose deux contraintes fortes : la latence doit etre faible, et l'etat doit rester coherent entre plusieurs utilisateurs connectes en meme temps.",
    }),
  );

  // 4. Architecture
  composeSlide(
    presentation,
    P.bg2,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 30 }, [
      title("Architecture globale", "REST structure les donnees. WebSocket porte le temps reel."),
      grid({ width: fill, height: fill, columns: [fr(0.8), fr(0.12), fr(0.9), fr(0.12), fr(0.9)], columnGap: 18 }, [
        column({ width: fill, height: fill, gap: 18, justify: "center" }, [
          card("Client", "Next.js 16\nReact 19\nTypeScript\nElectron", P.blue, { bodySize: 26 }),
        ]),
        column({ width: fill, height: fill, justify: "center", align: "center" }, [
          t("REST\n+\nWS", { size: 22, bold: true, color: P.cyan, width: hug }),
        ]),
        column({ width: fill, height: fill, gap: 18, justify: "center" }, [
          card("Backend", "Rust / Axum\nTokio\nTower-HTTP\nJWT + Argon2id", P.violet, { bodySize: 26 }),
        ]),
        column({ width: fill, height: fill, justify: "center", align: "center" }, [
          t("SQL\n+\nDocs", { size: 22, bold: true, color: P.green, width: hug }),
        ]),
        column({ width: fill, height: fill, gap: 18, justify: "center" }, [
          card("Data", "PostgreSQL\nUsers / roles / invites\n\nMongoDB\nMessages / DMs / reactions", P.green, { bodySize: 24 }),
        ]),
      ]),
      row({ width: fill, height: hug, gap: 20 }, [
        badge("REST : CRUD", P.blue),
        badge("WS : instantane", P.cyan),
        badge("BDD : roles separes", P.green),
      ]),
    ]),
    note({
      objective: "Donner au jury la carte mentale de l'architecture.",
      visual: "Schema horizontal client -> backend -> bases, avec deux canaux REST et WebSocket.",
      oral:
        "L'architecture est separee en couches. Le client Next.js communique avec le backend Rust de deux facons. REST est utilise pour les operations classiques : authentification, gestion des serveurs, channels, membres et invitations. WebSocket est utilise pour ce qui doit etre instantane : messages, typing, presence, reactions et DMs. Le backend distribue ensuite les donnees entre PostgreSQL et MongoDB selon leur nature.",
    }),
  );

  // 5. Backend
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Backend Rust / Axum", "Performance, securite memoire et serveur asynchrone."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1)], columnGap: 42 }, [
        column({ width: fill, height: fill, gap: 20, justify: "center" }, [
          bullet("Rust : pas de garbage collector, latence stable", P.blue),
          bullet("Axum 0.7 : routes REST + upgrade WebSocket", P.violet),
          bullet("Tokio : runtime asynchrone", P.cyan),
          bullet("Tower-HTTP : CORS, tracing, fichiers statiques", P.green),
          bullet("JWT + Argon2id : auth stateless et hash robuste", P.amber),
        ]),
        column({ width: fill, height: fill, gap: 16, justify: "center" }, [
          stackLabel("main.rs : Router Axum", P.violet),
          stackLabel("routes/* : API REST", P.blue),
          stackLabel("auth/* : JWT + Argon2id", P.amber),
          stackLabel("state.rs : AppState partage", P.cyan),
          stackLabel("db + repositories : persistance", P.green),
        ]),
      ]),
      t("Preuves : backend/Cargo.toml, backend/src/main.rs, backend/src/auth/*", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Justifier le choix du backend et des briques Rust.",
      visual: "A gauche les avantages, a droite les modules concrets du repo.",
      oral:
        "Le backend est le coeur du projet. Rust est pertinent parce qu'un chat temps reel garde beaucoup de connexions ouvertes. Sans garbage collector, on evite les pauses imprevisibles. Axum fournit le routage HTTP et l'upgrade WebSocket. Tokio execute les taches asynchrones. Tower-HTTP ajoute les couches transverses comme CORS, tracing et service de fichiers uploades. Cote securite, les JWT authentifient les requetes et Argon2id protege les mots de passe.",
    }),
  );

  // 6. WebSocket
  composeSlide(
    presentation,
    P.bg2,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 32 }, [
      title("WebSocket temps reel", "Un Hub custom garde les connexions et diffuse les evenements."),
      grid({ width: fill, height: fill, columns: [fr(1.05), fr(0.95)], columnGap: 50 }, [
        panel(
          {
            width: fill,
            height: fill,
            padding: { x: 34, y: 34 },
            borderRadius: "rounded-lg",
            fill: P.surface3,
            line: { color: P.cyan, width: 1 },
          },
          column({ width: fill, height: fill, gap: 16, justify: "center" }, [
            t("Hub WebSocket", { size: 44, bold: true, fontFace: FONT, color: P.cyan, width: fill }),
            rule({ width: fixed(240), stroke: P.cyan, weight: 4 }),
            stackLabel("connections : user -> connIds", P.blue),
            stackLabel("sockets : connId -> sender", P.violet),
            stackLabel("rooms : roomId -> connIds", P.green),
            stackLabel("heartbeats : connId -> time", P.amber),
          ]),
        ),
        column({ width: fill, height: fill, gap: 20, justify: "center" }, [
          bullet("DashMap : structures concurrentes sans Mutex global", P.cyan),
          bullet("Rooms : broadcast cible par canal ou DM", P.green),
          bullet("Typing + presence + heartbeat", P.blue),
          bullet("Messages, reactions, pins et DMs", P.violet),
          bullet("Protocole JSON type/payload valide cote front", P.amber),
        ]),
      ]),
      t("Preuves : backend/src/ws/hub.rs, ws/connection.rs, ws/protocol.rs, frontend/lib/ws/types.ts", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Faire comprendre la couche temps reel, centrale dans un chat.",
      visual: "Hub central avec ses quatre tables internes, puis usages a droite.",
      oral:
        "Le Hub WebSocket est la piece centrale du temps reel. Il conserve les connexions actives, les sockets d'envoi, les rooms et les derniers heartbeats. DashMap permet de manipuler ces structures de maniere concurrente sans mettre un gros verrou global. Quand un utilisateur envoie un message, le backend verifie le payload, persiste le message dans MongoDB, puis diffuse l'evenement aux connexions presentes dans la room concernee.",
    }),
  );

  // 7. Databases
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Bases de donnees", "PostgreSQL et MongoDB n'ont pas le meme role."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1)], columnGap: 34 }, [
        panel(
          {
            width: fill,
            height: fill,
            padding: { x: 34, y: 30 },
            borderRadius: "rounded-lg",
            fill: P.surface,
            line: { color: P.green, width: 1 },
          },
          column({ width: fill, height: fill, gap: 18 }, [
            t("PostgreSQL 16", { size: 42, bold: true, fontFace: FONT, color: P.green, width: fill }),
            bullet("Users, servers, channels", P.green),
            bullet("Memberships, roles, invites, bans", P.green),
            bullet("Contraintes relationnelles et coherence", P.green),
            bullet("SQLx 0.8 pour requetes typees", P.green),
          ]),
        ),
        panel(
          {
            width: fill,
            height: fill,
            padding: { x: 34, y: 30 },
            borderRadius: "rounded-lg",
            fill: P.surface,
            line: { color: P.amber, width: 1 },
          },
          column({ width: fill, height: fill, gap: 18 }, [
            t("MongoDB 6", { size: 42, bold: true, fontFace: FONT, color: P.amber, width: fill }),
            bullet("Messages et DMs", P.amber),
            bullet("Reactions, pins, historique", P.amber),
            bullet("Documents flexibles et ecritures frequentes", P.amber),
            bullet("Driver mongodb 2.8 cote Rust", P.amber),
          ]),
        ),
      ]),
      t("Preuves : backend/docker-compose.yml, backend/database/migrations/*, backend/src/db/message_repo.rs", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Justifier l'architecture polyglotte de persistance.",
      visual: "Tableau comparatif en deux colonnes, role clair pour chaque base.",
      oral:
        "Le choix PostgreSQL plus MongoDB est intentionnel. PostgreSQL gere les donnees structurées avec des relations fortes : utilisateurs, serveurs, canaux, memberships et droits. MongoDB gere les messages, parce que leur structure est plus flexible et evolue avec les reactions, pins, pieces jointes ou messages directs. Cela evite de forcer tous les cas d'usage dans un seul modele de donnees.",
    }),
  );

  // 8. Frontend
  composeSlide(
    presentation,
    P.bg2,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Frontend Next.js / React / TypeScript", "Un client typé, modulaire et valide aux frontieres."),
      grid({ width: fill, height: fill, columns: [fr(0.95), fr(1.05)], columnGap: 48 }, [
        column({ width: fill, height: fill, gap: 18, justify: "center" }, [
          stackLabel("Next.js 16 - App Router", P.blue),
          stackLabel("React 19 - UI composants", P.violet),
          stackLabel("TypeScript - typage statique", P.cyan),
          stackLabel("Zustand - 11 stores", P.green),
          stackLabel("Zod - validation API + WS", P.amber),
        ]),
        column({ width: fill, height: fill, gap: 20, justify: "center" }, [
          bullet("Rewrites /api vers le backend Rust", P.blue),
          bullet("Stores separes : auth, messages, DM, presence, typing", P.green),
          bullet("Zod parse les reponses API et ServerEventSchema", P.amber),
          bullet("Tailwind CSS 4 + shadcn/ui + Radix", P.violet),
          bullet("lucide-react, sonner, emoji-mart, framer-motion", P.cyan),
        ]),
      ]),
      t("Preuves : frontend/real-time-chat/package.json, next.config.ts, store/*, lib/ws/types.ts", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Montrer que le frontend est structure et securise cote contrat de donnees.",
      visual: "Pile technique a gauche, role applicatif a droite.",
      oral:
        "Le frontend utilise Next.js 16, React 19 et TypeScript. Next.js structure les routes et redirige les appels /api vers le backend Rust. Zustand gere l'etat global avec des stores specialises, ce qui evite un gros store monolithique. Zod est important : il valide les donnees recues du backend et les evenements WebSocket avant qu'ils entrent dans l'application. Cela rend le frontend plus robuste face aux changements de contrat.",
    }),
  );

  // 9. Desktop
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Desktop Electron", "Un client desktop sans dupliquer l'interface web."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1)], columnGap: 56 }, [
        column({ width: fill, height: fill, justify: "center", gap: 16 }, [
          card("Web app", "Next.js charge l'experience EpiTalk", P.blue, { height: fixed(140) }),
          t("->", { size: 54, bold: true, color: P.cyan, width: fill }),
          card("Electron", "BrowserWindow + menu + notifications natives", P.violet, { height: fixed(160) }),
        ]),
        column({ width: fill, height: fill, gap: 22, justify: "center" }, [
          bullet("Electron 32 wrappe l'URL de l'application web", P.violet),
          bullet("Zero duplication de code frontend", P.blue),
          bullet("Notifications systeme via API Notification", P.cyan),
          bullet("Menu applicatif localise fr/en", P.green),
          bullet("contextIsolation: true", P.amber),
        ]),
      ]),
      t("Preuves : desktop-app/package.json, desktop-app/main.js, desktop-app/src/notifications.js", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Expliquer la valeur d'Electron dans le projet.",
      visual: "Schema de reutilisation : web app vers shell desktop.",
      oral:
        "L'application desktop est pragmatique : elle ne recree pas l'interface, elle charge l'application web dans une BrowserWindow Electron. On garde donc une seule base de code UI pour le web et le desktop. Electron ajoute ce que le navigateur ne donne pas aussi naturellement dans ce contexte : notifications systeme, menu applicatif, integration desktop et focus de la fenetre quand l'utilisateur clique une notification.",
    }),
  );

  // 10. Infra CI
  composeSlide(
    presentation,
    P.bg2,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Infrastructure / CI-CD", "Docker pour l'environnement, GitHub Actions pour la qualite et l'image."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1.1)], columnGap: 48 }, [
        column({ width: fill, height: fill, gap: 16, justify: "center" }, [
          stackLabel("Docker Compose", P.blue),
          stackLabel("Backend Rust", P.violet),
          stackLabel("PostgreSQL 16 + MongoDB 6", P.green),
          stackLabel("GitHub Actions", P.cyan),
          stackLabel("GHCR", P.amber),
        ]),
        column({ width: fill, height: fill, gap: 20, justify: "center" }, [
          bullet("docker-compose : backend, PostgreSQL, MongoDB, Adminer", P.blue),
          bullet("Dockerfile multi-stage : rust:1.84-slim -> debian slim", P.violet),
          bullet("CI : rustfmt, clippy, build release, tests avec BDD", P.cyan),
          bullet("Deploy : build image Docker et push vers GHCR", P.amber),
          bullet("Workflows : ci.yml, deploy.yml, ws_test.yml", P.green),
        ]),
      ]),
      t("Preuves : backend/Dockerfile, backend/docker-compose.yml, .github/workflows/*", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Montrer que le projet est industrialise au-dela du code applicatif.",
      visual: "Pipeline lisible de Docker Compose vers GitHub Actions et GHCR.",
      oral:
        "L'infrastructure locale repose sur Docker Compose : backend, PostgreSQL, MongoDB et Adminer. Le Dockerfile est multi-stage, ce qui permet de compiler dans une image Rust puis d'executer dans une image Debian slim. Cote CI, GitHub Actions lance le formatage, clippy, le build, les tests backend avec PostgreSQL et MongoDB en services, et un workflow dedie aux tests WebSocket. Le deploiement pousse l'image vers GHCR, mais le deploiement production final reste a completer.",
    }),
  );

  // 11. Tests
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Tests et qualite", "La qualite couvre backend, frontend, desktop et temps reel."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1), fr(1)], columnGap: 24 }, [
        card("Backend Rust", "Tests unitaires\nIntegration tests\nPostgreSQL + MongoDB", P.violet, { height: fill, bodySize: 25 }),
        card("WebSocket", "Binaire ws_test\nFlow typing / presence\nMessages et DMs", P.cyan, { height: fill, bodySize: 25 }),
        card("Frontend + Desktop", "Vitest 4\nTesting Library\ncoverage-v8", P.blue, { height: fill, bodySize: 25 }),
      ]),
      row({ width: fill, height: hug, gap: 20 }, [
        badge("rustfmt", P.green),
        badge("clippy -D warnings", P.green),
        badge("cargo test", P.green),
        badge("vitest", P.green),
      ]),
      t("Preuves : backend/src/tests/*, integration_tests.rs, frontend vitest configs, desktop-app/package.json", { size: 15, color: P.dim, width: fill }),
    ]),
    note({
      objective: "Rassurer le jury sur le niveau de verification.",
      visual: "Trois colonnes : backend, WebSocket, frontend/desktop.",
      oral:
        "La qualite est testee a plusieurs niveaux. Cote backend, il y a des tests unitaires dans les modules et des tests d'integration avec de vraies bases PostgreSQL et MongoDB en CI. Le WebSocket a un binaire de test dedie et des flows pour la presence, le typing et les messages. Cote frontend et desktop, le projet utilise Vitest, Testing Library et coverage-v8.",
    }),
  );

  // 12. Limits
  composeSlide(
    presentation,
    P.bg2,
    column({ width: fill, height: fill, padding: { x: 92, y: 70 }, gap: 34 }, [
      title("Limites et ameliorations", "Les limites sont identifiees et transformables en plan d'action."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1)], rows: [auto, auto], columnGap: 28, rowGap: 28 }, [
        card("Deploiement prod", "GHCR est pret, orchestration finale non configuree", P.amber),
        card("Migrations", "Numerotation 004_* a clarifier", P.red),
        card("Scalabilite WebSocket", "Hub single-node : prevoir Redis pub/sub", P.cyan),
        card("Lockfiles frontend", "pnpm-lock.yaml + package-lock a nettoyer", P.violet),
      ]),
      t("A presenter comme des perspectives, pas comme des faiblesses cachees.", { size: 23, color: P.muted, width: fill }),
    ]),
    note({
      objective: "Montrer une prise de recul honnete et professionnelle.",
      visual: "Quatre limites formulees comme chantiers concrets.",
      oral:
        "Les limites principales sont clairement identifiees. Le deploiement pousse l'image Docker vers GHCR, mais l'orchestration production n'est pas encore finalisee. Les migrations ont plusieurs fichiers en 004, donc il faut clarifier l'ordre. Le Hub WebSocket est en memoire sur un seul noeud : pour scaler horizontalement, il faudra un bus comme Redis Pub/Sub. Enfin, les lockfiles frontend indiquent un workflow package manager a nettoyer.",
    }),
  );

  // 13. Conclusion
  composeSlide(
    presentation,
    P.bg,
    column({ width: fill, height: fill, padding: { x: 92, y: 74 }, gap: 38 }, [
      title("Conclusion", "Une stack coherente avec les contraintes d'un chat temps reel."),
      grid({ width: fill, height: fill, columns: [fr(1), fr(1), fr(1)], columnGap: 28 }, [
        card("Robuste", "Rust, typage fort, tests, CI et validations Zod", P.violet, { height: fill }),
        card("Temps reel", "WebSocket Hub, rooms, presence, typing et heartbeat", P.cyan, { height: fill }),
        card("Evolutive", "Separation REST / WS, PostgreSQL / MongoDB, Docker", P.green, { height: fill }),
      ]),
      panel(
        {
          width: fill,
          height: hug,
          padding: { x: 30, y: 22 },
          borderRadius: "rounded-lg",
          fill: "#2F80FF18",
          line: { color: "#2F80FF88", width: 1 },
        },
        t("Message final : la stack n'est pas un assemblage de modes, elle repond aux besoins du produit.", {
          size: 30,
          bold: true,
          color: P.ink,
          width: fill,
        }),
      ),
    ]),
    note({
      objective: "Finir sur la coherence de l'architecture et les points forts.",
      visual: "Trois piliers : robuste, temps reel, evolutive.",
      oral:
        "Pour conclure, la stack d'EpiTalk est coherente avec le produit. Rust et Axum donnent un backend performant et fiable. Le WebSocket apporte l'instantaneite attendue dans un chat. PostgreSQL et MongoDB se completent au lieu de se concurrencer. Next.js, React, TypeScript, Zustand et Zod structurent un frontend maintenable. Enfin, Docker et GitHub Actions donnent une base d'industrialisation. Le projet a encore des axes d'amelioration, mais ils sont identifies, limites et compatibles avec l'architecture actuelle.",
    }),
  );

  return presentation;
}

async function saveBlob(blob, filePath) {
  const bytes = Buffer.from(await blob.arrayBuffer());
  await fs.writeFile(filePath, bytes);
}

async function main() {
  await fs.mkdir(OUT, { recursive: true });
  await fs.mkdir(SCRATCH, { recursive: true });

  // Clean deliverable directory for the presentation skill output hygiene check.
  for (const file of await fs.readdir(OUT)) {
    await fs.rm(path.join(OUT, file), { recursive: true, force: true });
  }

  const deck = createDeck();
  const pptx = await PresentationFile.exportPptx(deck);
  const final = path.join(OUT, "output.pptx");
  await pptx.save(final);

  const previewDir = path.join(SCRATCH, "previews");
  await fs.mkdir(previewDir, { recursive: true });
  for (const file of await fs.readdir(previewDir)) {
    await fs.rm(path.join(previewDir, file), { recursive: true, force: true });
  }

  const layoutReports = [];
  for (let i = 0; i < deck.slides.count; i += 1) {
    const slide = deck.slides.getItem(i);
    const png = await slide.export({ format: "png" });
    await saveBlob(png, path.join(previewDir, `slide-${String(i + 1).padStart(2, "0")}.png`));
    const layout = await slide.export({ format: "layout" });
    layoutReports.push(layout);
  }
  await fs.writeFile(path.join(SCRATCH, "layout-report.json"), JSON.stringify(layoutReports, null, 2), "utf8");

  const notesMd = [
    "# EpiTalk - Presentation Stack Technique",
    "",
    "## Pitch oral - 2 minutes",
    "",
    pitch,
    "",
    "## Plan global",
    "",
    plan,
    "",
    "## Mini-checklist avant soutenance",
    "",
    "- Lancer une fois le pitch au chronometre : viser 1 min 50 a 2 min 10.",
    "- Preparer une reponse courte sur Rust vs Node/Go.",
    "- Verifier le vocabulaire : Argon2id, JWT HS256, WebSocket Hub, SQLx, Zod.",
    "- Assumer clairement les limites : prod non finalisee, migrations, scaling WebSocket, lockfiles.",
    "- Garder les slides courtes : les details sont dans les notes orales.",
  ].join("\n");
  await fs.writeFile(path.join(SCRATCH, "powerpoint-ready-notes.md"), notesMd, "utf8");

  console.log(JSON.stringify({ final, previewDir, slides: deck.slides.count }, null, 2));
}

await main();

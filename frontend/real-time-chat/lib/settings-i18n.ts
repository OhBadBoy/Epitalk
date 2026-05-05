import type { Language } from "./i18n";

export function getAuthHomeLabel(language: Language): string {
  return language === "en" ? "Back to home" : "Retour a l'accueil";
}

export function getSettingsNavLabel(
  language: Language,
  section: "profile" | "appearance" | "privacy" | "notifications",
): string {
  const labels = {
    fr: {
      profile: "Profil",
      appearance: "Apparence",
      privacy: "Confidentialite",
      notifications: "Notifications",
    },
    en: {
      profile: "Profile",
      appearance: "Appearance",
      privacy: "Privacy",
      notifications: "Notifications",
    },
  } as const;

  return labels[language][section];
}

export function getSettingsStatusLabel(
  language: Language,
  status: "ONLINE" | "IDLE" | "DND" | "OFFLINE",
): string {
  const labels = {
    fr: {
      ONLINE: "Actif",
      IDLE: "Inactif",
      DND: "Silencieux",
      OFFLINE: "Hors ligne",
    },
    en: {
      ONLINE: "Active",
      IDLE: "Idle",
      DND: "Do not disturb",
      OFFLINE: "Offline",
    },
  } as const;

  return labels[language][status];
}

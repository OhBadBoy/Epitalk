export type Language = "fr" | "en";

export const DEFAULT_LANGUAGE: Language = "fr";

export const translations = {
  fr: {
    nav: {
      about: "À Propos",
      features: "Fonctionnalités",
      solution: "Solution",
      pricing: "Tarifs",
      signIn: "Connexion",
      signUp: "Inscription",
      joinServer: "Rejoindre un serveur",
    },
    hero: {
      title: "Connecter, discuter, partager.",
      subtitle:
        "EpiTalk est une plateforme de discussion en temps réel pensée pour créer, organiser et faire grandir des communautés modernes.",
      ctaPrimary: "Créer mon premier serveur",
    },
    stats: {
      activitiesTitle: "Activités",
      currentYearDescription:
        "Cette année, les échanges sur EpiTalk sont plus actifs que l’an dernier.",
      messagesPerDay: "messages / jour",
      currentYearLabel: "2026",
      previousYearLabel: "2025",
    },
  },
  en: {
    nav: {
      about: "About",
      features: "Features",
      solution: "Solution",
      pricing: "Pricing",
      signIn: "Sign in",
      signUp: "Sign up",
      joinServer: "Join a server",
    },
    hero: {
      title: "Connect, chat, share.",
      subtitle:
        "EpiTalk is a real-time messaging platform designed to create, organize, and grow modern communities.",
      ctaPrimary: "Create my first server",
    },
    stats: {
      activitiesTitle: "Activity",
      currentYearDescription:
        "This year, conversations on EpiTalk are more active than last year.",
      messagesPerDay: "messages / day",
      currentYearLabel: "2026",
      previousYearLabel: "2025",
    },
  },
} as const;

export type Translations = (typeof translations)[Language];

export function getTranslations(language: Language): Translations {
  if (language in translations) {
    return translations[language as keyof typeof translations] as Translations;
  }

  return translations[DEFAULT_LANGUAGE] as Translations;
}

const DEFAULT_LANGUAGE = "fr";

function detectLanguage(env = process.env) {
  const fromEnv = (
    env.EPITALK_LANG ||
    env.LC_ALL ||
    env.LC_MESSAGES ||
    env.LANGUAGE ||
    env.LANG ||
    ""
  ).toLowerCase();

  if (fromEnv.startsWith("fr")) return "fr";
  if (fromEnv.startsWith("en")) return "en";

  return DEFAULT_LANGUAGE;
}

const messages = {
  fr: {
    appLabel: "EpiTalk",
    quitLabel: "Quitter EpiTalk",
    viewLabel: "Affichage",
    reloadLabel: "Recharger",
    toggleDevToolsLabel: "Basculer les outils de développement",
    readyTitle: (appName) => `${appName} est prêt`,
    readyBody: "Votre application de chat est maintenant disponible.",
  },
  en: {
    appLabel: "EpiTalk",
    quitLabel: "Quit EpiTalk",
    viewLabel: "View",
    reloadLabel: "Reload",
    toggleDevToolsLabel: "Toggle Developer Tools",
    readyTitle: (appName) => `${appName} is ready`,
    readyBody: "Your chat application is now available.",
  },
};

function getTranslations(lang = detectLanguage()) {
  if (lang in messages) {
    return messages[lang];
  }
  return messages[DEFAULT_LANGUAGE];
}

module.exports = {
  DEFAULT_LANGUAGE,
  detectLanguage,
  getTranslations,
};

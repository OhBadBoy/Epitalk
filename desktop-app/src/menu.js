const { getTranslations } = require("./i18n");

function buildMenuTemplate(lang) {
  const t = getTranslations(lang);

  return [
    {
      label: t.appLabel,
      submenu: [
        {
          role: "quit",
          label: t.quitLabel,
        },
      ],
    },
    {
      label: t.viewLabel,
      submenu: [
        {
          role: "reload",
          label: t.reloadLabel,
        },
        {
          role: "toggleDevTools",
          label: t.toggleDevToolsLabel,
        },
      ],
    },
  ];
}

module.exports = {
  buildMenuTemplate,
};

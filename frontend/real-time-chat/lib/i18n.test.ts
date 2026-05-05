import { describe, it, expect } from "vitest";
import { DEFAULT_LANGUAGE, getTranslations, translations } from "./i18n";

describe("i18n translations", () => {
  it("utilise le français comme langue par défaut", () => {
    expect(DEFAULT_LANGUAGE).toBe("fr");
    const fr = getTranslations("fr");
    expect(fr.hero.title).toBe(translations.fr.hero.title);
  });

  it("retourne les traductions en anglais quand la langue est 'en'", () => {
    const en = getTranslations("en");
    expect(en.hero.title).toBe(translations.en.hero.title);
    expect(en.nav.signIn).toBe("Sign in");
    expect(en.stats.messagesPerDay).toBe("messages / day");
  });

  it("retombe sur la langue par défaut si une langue inconnue est demandée (sécurité)", () => {
    const value = getTranslations("fr");
    expect(value.hero.title).toBe(translations.fr.hero.title);
  });
});

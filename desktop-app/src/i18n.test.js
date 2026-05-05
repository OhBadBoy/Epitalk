import { describe, it, expect } from "vitest";
import { detectLanguage, getTranslations } from "./i18n";

describe("i18n.detectLanguage", () => {
  it("prioritizes EPITALK_LANG when provided", () => {
    expect(detectLanguage({ EPITALK_LANG: "en_US.UTF-8", LANG: "fr_FR.UTF-8" })).toBe("en");
  });

  it("detects english from locale-related environment variables", () => {
    expect(detectLanguage({ LC_MESSAGES: "en_US.UTF-8" })).toBe("en");
    expect(detectLanguage({ LANGUAGE: "en_US:en" })).toBe("en");
  });

  it("falls back to french when nothing is set", () => {
    expect(detectLanguage({})).toBe("fr");
  });
});

describe("i18n.getTranslations", () => {
  it("returns english strings for en language", () => {
    const t = getTranslations("en");
    expect(t.viewLabel).toBe("View");
  });
});

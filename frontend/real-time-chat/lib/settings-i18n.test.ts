import { describe, expect, it } from "vitest";
import {
  getAuthHomeLabel,
  getSettingsNavLabel,
  getSettingsStatusLabel,
} from "./settings-i18n";

describe("settings-i18n", () => {
  it("returns localized auth back-home label", () => {
    expect(getAuthHomeLabel("en")).toBe("Back to home");
    expect(getAuthHomeLabel("fr")).toBe("Retour a l'accueil");
  });

  it("returns localized settings navigation labels", () => {
    expect(getSettingsNavLabel("en", "profile")).toBe("Profile");
    expect(getSettingsNavLabel("fr", "appearance")).toBe("Apparence");
  });

  it("returns localized settings status labels", () => {
    expect(getSettingsStatusLabel("en", "OFFLINE")).toBe("Offline");
    expect(getSettingsStatusLabel("fr", "DND")).toBe("Silencieux");
  });
});

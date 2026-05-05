import { describe, it, expect } from "vitest";
import { getReadyNotificationOptions, shouldShowReadyNotification } from "./notifications";

describe("notifications.getReadyNotificationOptions", () => {
  it("retourne un titre et un corps par défaut pour EpiTalk en français", () => {
    const opts = getReadyNotificationOptions("EpiTalk", "fr");
    expect(opts.title).toBe("EpiTalk est prêt");
    expect(opts.body).toContain("chat");
  });

  it("utilise le nom d'application fourni dans le titre en français", () => {
    const opts = getReadyNotificationOptions("MonApp", "fr");
    expect(opts.title).toBe("MonApp est prêt");
  });

  it("peut retourner un message en anglais", () => {
    const opts = getReadyNotificationOptions("EpiTalk", "en");
    expect(opts.title).toBe("EpiTalk is ready");
    expect(opts.body.toLowerCase()).toContain("chat");
  });
});

describe("notifications.shouldShowReadyNotification", () => {
  it("retourne true uniquement au premier lancement sans focus", () => {
    expect(shouldShowReadyNotification({ isFirstLaunch: true, hasFocus: false })).toBe(true);
    expect(shouldShowReadyNotification({ isFirstLaunch: false, hasFocus: false })).toBe(false);
    expect(shouldShowReadyNotification({ isFirstLaunch: true, hasFocus: true })).toBe(false);
  });
});

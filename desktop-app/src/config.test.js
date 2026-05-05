import { describe, it, expect } from "vitest";
import { getWebAppUrl } from "./config";

describe("config.getWebAppUrl", () => {
  it("retourne l'URL par défaut quand aucune variable n'est définie", () => {
    const url = getWebAppUrl({});
    expect(url).toBe("http://localhost:3000");
  });

  it("utilise EPITALK_WEB_URL quand elle est valide (http)", () => {
    const url = getWebAppUrl({ EPITALK_WEB_URL: "http://example.com" });
    expect(url).toBe("http://example.com");
  });

  it("utilise EPITALK_WEB_URL quand elle est valide (https)", () => {
    const url = getWebAppUrl({ EPITALK_WEB_URL: "https://secure.example.com" });
    expect(url).toBe("https://secure.example.com");
  });

  it("ignore une EPITALK_WEB_URL invalide et retombe sur la valeur par défaut", () => {
    const url = getWebAppUrl({ EPITALK_WEB_URL: "example.com" });
    expect(url).toBe("http://localhost:3000");
  });
});

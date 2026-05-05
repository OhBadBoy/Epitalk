import { describe, it, expect } from "vitest";
import { buildMenuTemplate } from "./menu";

describe("menu.buildMenuTemplate", () => {
  it("définit un menu EpiTalk avec quitter en français par défaut", () => {
    const template = buildMenuTemplate("fr");
    const appMenu = template[0];

    expect(appMenu.label).toBe("EpiTalk");
    expect(appMenu.submenu?.some((item) => item.role === "quit")).toBe(true);
  });

  it("définit un menu Affichage avec reload et devtools en français", () => {
    const template = buildMenuTemplate("fr");
    const viewMenu = template[1];

    const roles = (viewMenu.submenu ?? []).map((item) => item.role);
    expect(roles).toContain("reload");
    expect(roles).toContain("toggleDevTools");
  });

  it("permet de construire un menu en anglais", () => {
    const template = buildMenuTemplate("en");
    const appMenu = template[0];
    const viewMenu = template[1];

    expect(appMenu.label).toBe("EpiTalk");
    expect(viewMenu.label).toBe("View");
  });
});

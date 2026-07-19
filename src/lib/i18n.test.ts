import { describe, it, expect, beforeEach } from "vitest";
import { t, getLang, setLang, toggleLang } from "./i18n";

beforeEach(() => {
  setLang("zh");
});

describe("i18n", () => {
  it("t() returns correct Chinese translation", () => {
    expect(t("nav.dashboard")).toBe("仪表盘");
  });

  it("t() returns key when key does not exist", () => {
    expect(t("nonexistent.key")).toBe("nonexistent.key");
  });

  it("getLang() returns current language", () => {
    expect(getLang()).toBe("zh");
    setLang("en");
    expect(getLang()).toBe("en");
  });

  it("setLang() switches language and t() returns the other language", () => {
    setLang("en");
    expect(t("nav.dashboard")).toBe("Dashboard");
  });

  it("setLang() ignores invalid language", () => {
    setLang("fr");
    expect(getLang()).toBe("zh");
  });

  it("toggleLang() switches between zh and en", () => {
    expect(getLang()).toBe("zh");
    toggleLang();
    expect(getLang()).toBe("en");
    toggleLang();
    expect(getLang()).toBe("zh");
  });
});

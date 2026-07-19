import { describe, it, expect } from "vitest";
import { CATEGORIES, JOURNAL_CATALOG } from "./journal-catalog";

describe("journal-catalog", () => {
  it("every journal belongs to a declared category", () => {
    const categories = new Set<string>(CATEGORIES);
    for (const j of JOURNAL_CATALOG) {
      expect(categories.has(j.category), `${j.name} has unknown category ${j.category}`).toBe(true);
    }
  });

  it("every journal has a name, ISSN and an http(s) RSS URL", () => {
    for (const j of JOURNAL_CATALOG) {
      expect(j.name.trim().length).toBeGreaterThan(0);
      expect(j.issn.trim().length).toBeGreaterThan(0);
      expect(j.rssUrl).toMatch(/^https?:\/\//);
    }
  });

  it("has no duplicate journals or RSS URLs", () => {
    const names = JOURNAL_CATALOG.map((j) => j.name);
    const urls = JOURNAL_CATALOG.map((j) => j.rssUrl);
    expect(new Set(names).size).toBe(names.length);
    expect(new Set(urls).size).toBe(urls.length);
  });

  it("every declared category contains at least one journal", () => {
    for (const c of CATEGORIES) {
      expect(
        JOURNAL_CATALOG.some((j) => j.category === c),
        `category ${c} is empty`
      ).toBe(true);
    }
  });
});

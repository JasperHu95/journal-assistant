import { describe, it, expect } from "vitest";
import { formatDate } from "./db";

describe("formatDate", () => {
  it("returns empty string for null/undefined/empty", () => {
    expect(formatDate(null)).toBe("");
    expect(formatDate(undefined)).toBe("");
    expect(formatDate("")).toBe("");
  });

  it("returns formatted string for valid ISO date", () => {
    const iso = "2003-03-04T04:05:06Z";
    const result = formatDate(iso);
    expect(result).not.toBe("");
    expect(result).not.toBe(iso);
    expect(result).toContain("2003");
  });

  it("returns original string for invalid date", () => {
    expect(formatDate("not-a-date")).toBe("not-a-date");
  });
});

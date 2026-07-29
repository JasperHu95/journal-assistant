import { describe, it, expect } from "vitest";
import { formatDate, formatRelativeTime } from "./db";

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

describe("formatRelativeTime", () => {
  it("returns empty string for null/undefined/empty", () => {
    expect(formatRelativeTime(null)).toBe("");
    expect(formatRelativeTime(undefined)).toBe("");
    expect(formatRelativeTime("")).toBe("");
  });

  it("returns 刚刚 for less than 1 minute ago", () => {
    const iso = new Date(Date.now() - 30 * 1000).toISOString();
    expect(formatRelativeTime(iso)).toBe("刚刚");
  });

  it("returns N 分钟前 for 1-59 minutes ago", () => {
    const iso = new Date(Date.now() - 5 * 60000).toISOString();
    expect(formatRelativeTime(iso)).toBe("5 分钟前");
  });

  it("returns N 小时前 for 1-23 hours ago", () => {
    const iso = new Date(Date.now() - 3 * 3600000).toISOString();
    expect(formatRelativeTime(iso)).toBe("3 小时前");
  });

  it("returns N 天前 for 1-29 days ago", () => {
    const iso = new Date(Date.now() - 5 * 86400000).toISOString();
    expect(formatRelativeTime(iso)).toBe("5 天前");
  });

  it("returns full date for 30+ days ago", () => {
    const iso = new Date(Date.now() - 40 * 86400000).toISOString();
    expect(formatRelativeTime(iso)).toBe(formatDate(iso));
  });

  it("returns original string for invalid date", () => {
    expect(formatRelativeTime("not-a-date")).toBe("not-a-date");
  });
});

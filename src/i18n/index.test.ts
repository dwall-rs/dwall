import { afterEach, describe, expect, it, vi } from "vitest";

import { matchLocale } from "./index";

describe("matchLocale", () => {
  it("resolves a bare language tag to the dictionary that exists", () => {
    expect(matchLocale("en")).toBe("en-US");
    expect(matchLocale("en-GB")).toBe("en-US");
    expect(matchLocale("ja")).toBe("ja-JP");
    expect(matchLocale("ja-JP")).toBe("ja-JP");
    expect(matchLocale("ko")).toBe("ko-KR");
    expect(matchLocale("zh")).toBe("zh-CN");
  });

  it("keeps the Chinese variants apart", () => {
    expect(matchLocale("zh-CN")).toBe("zh-CN");
    expect(matchLocale("zh-Hans-CN")).toBe("zh-CN");
    expect(matchLocale("zh-TW")).toBe("zh-TW");
    expect(matchLocale("zh-Hant")).toBe("zh-TW");
    expect(matchLocale("zh-HK")).toBe("zh-HK");
    expect(matchLocale("zh-Hant-HK")).toBe("zh-HK");
    expect(matchLocale("zh-MO")).toBe("zh-HK");
  });

  it("has nothing to say about a language it does not carry", () => {
    expect(matchLocale("de-DE")).toBeUndefined();
    expect(matchLocale("fr")).toBeUndefined();
    expect(matchLocale("not a tag")).toBeUndefined();
  });
});

describe("the saved locale", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  /** The module reads storage at evaluation time, so it has to be installed first. */
  const installStorage = (values: Record<string, string>) => {
    const entries = new Map(Object.entries(values));
    vi.stubGlobal("localStorage", {
      getItem: (key: string) => entries.get(key) ?? null,
      setItem: (key: string, value: string) => entries.set(key, value),
      removeItem: (key: string) => entries.delete(key),
    });
  };

  it("ignores a stored value inherited from Object.prototype", async () => {
    installStorage({ "locale-choice": "1", locale: "toString" });

    const { locale } = await import("./index");

    expect(
      locale(),
      "a prototype property must not be treated as a language and crash the module",
    ).toBe("en-US");
  });

  it("still accepts a language the table really carries", async () => {
    installStorage({ "locale-choice": "1", locale: "ja-JP" });

    const { locale } = await import("./index");

    expect(locale()).toBe("ja-JP");
  });
});

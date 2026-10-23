import { describe, expect, it } from "vitest";

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

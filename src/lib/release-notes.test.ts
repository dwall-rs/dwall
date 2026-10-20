import { describe, expect, test } from "vitest";

import { parseReleaseNoteSpans, parseReleaseNotes } from "./release-notes";

describe("release notes: blocks", () => {
  test("a bullet list becomes one list block", () => {
    expect(parseReleaseNotes(["- one", "- two", "* three"].join("\n"))).toEqual(
      [{ kind: "list", items: ["one", "two", "three"] }],
    );
  });

  test("a blank line ends the list", () => {
    expect(parseReleaseNotes("- one\n\n- two")).toEqual([
      { kind: "list", items: ["one"] },
      { kind: "list", items: ["two"] },
    ]);
  });

  test("headings of any level become heading blocks", () => {
    expect(parseReleaseNotes("## Top\n### Deeper")).toEqual([
      { kind: "heading", text: "Top" },
      { kind: "heading", text: "Deeper" },
    ]);
  });

  /** `#hashtag` is text, not a heading: a heading needs the space. */
  test("a hash without a space stays a paragraph", () => {
    expect(parseReleaseNotes("#hashtag")).toEqual([
      { kind: "paragraph", text: "#hashtag" },
    ]);
  });

  test("plain lines become paragraphs and the list resumes after them", () => {
    expect(parseReleaseNotes("- one\nBoth are listed.\n- two")).toEqual([
      { kind: "list", items: ["one"] },
      { kind: "paragraph", text: "Both are listed." },
      { kind: "list", items: ["two"] },
    ]);
  });

  test("carriage returns are tolerated", () => {
    expect(parseReleaseNotes("- one\r\n- two\r\n")).toEqual([
      { kind: "list", items: ["one", "two"] },
    ]);
  });

  test("empty notes produce no blocks", () => {
    expect(parseReleaseNotes("")).toEqual([]);
    expect(parseReleaseNotes("\n\n")).toEqual([]);
  });
});

describe("release notes: spans", () => {
  test("backticks mark code", () => {
    expect(parseReleaseNoteSpans("run `dwall.exe` now")).toEqual([
      { code: false, text: "run " },
      { code: true, text: "dwall.exe" },
      { code: false, text: " now" },
    ]);
  });

  test("text without backticks is one plain span", () => {
    expect(parseReleaseNoteSpans("plain")).toEqual([
      { code: false, text: "plain" },
    ]);
  });

  /** An unbalanced backtick swallows the rest as code rather than dropping it. */
  test("an unbalanced backtick keeps the text visible", () => {
    expect(parseReleaseNoteSpans("a `b")).toEqual([
      { code: false, text: "a " },
      { code: true, text: "b" },
    ]);
  });

  test("empty spans are dropped", () => {
    expect(parseReleaseNoteSpans("`a``b`")).toEqual([
      { code: true, text: "a" },
      { code: true, text: "b" },
    ]);
  });
});

import { describe, expect, test } from "vitest";

import type { CatalogTheme } from "@/ipc";

import { themeById, themeList } from "./themes";

const entry = (over: Partial<CatalogTheme>): CatalogTheme => ({
  id: "Big Sur",
  name: "Big Sur",
  source: "catalog",
  thumbnails: [],
  installed: false,
  ...over,
});

describe("themeList", () => {
  test("marks a catalog theme whose frames are missing", () => {
    expect(themeList([entry({ installed: false })])[0].needsDownload).toBe(
      true,
    );
  });

  test("leaves an installed catalog theme alone", () => {
    expect(themeList([entry({ installed: true })])[0].needsDownload).toBe(
      false,
    );
  });

  test("never offers to download a custom theme, which is on disk already", () => {
    expect(
      themeList([entry({ source: "custom", installed: false })])[0]
        .needsDownload,
    ).toBe(false);
  });

  test("reuses the model while the entry's fields are unchanged", () => {
    // Row identity is what keeps <For> from remounting every tile (and refetching
    // every thumbnail) when the search box recomputes the list.
    const catalogEntry = entry({});

    expect(themeList([catalogEntry])[0]).toBe(themeList([catalogEntry])[0]);
  });

  test("rebuilds the model once the entry's installed flag flips", () => {
    const catalogEntry = entry({ installed: false });
    const before = themeList([catalogEntry])[0];
    expect(before.needsDownload).toBe(true);

    catalogEntry.installed = true;
    const after = themeList([catalogEntry])[0];

    expect(after.needsDownload).toBe(false);
    expect(after).not.toBe(before);
  });
});

describe("themeById", () => {
  test("takes the flag from the catalog entry", () => {
    expect(themeById([entry({})], "Big Sur").needsDownload).toBe(true);
  });

  test("does not invent a download for a theme outside the catalog", () => {
    expect(themeById([entry({})], "My Own Theme")).toEqual({
      id: "My Own Theme",
      name: "My Own Theme",
      needsDownload: false,
    });
  });

  test("falls back to the first catalog theme when no id is set", () => {
    const theme = themeById(
      [entry({ id: "Catalina", name: "Catalina" })],
      undefined,
    );
    expect(theme.id).toBe("Catalina");
  });
});

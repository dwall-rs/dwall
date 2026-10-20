import { beforeEach, describe, expect, test, vi } from "vitest";

/**
 * A stand-in for the webview's storage. The store reads it at module load, so each test
 * resets the modules and installs its own before importing.
 */
function fakeStorage(initial: Record<string, string> = {}) {
  const items = new Map(Object.entries(initial));

  return {
    items,
    getItem: vi.fn((key: string) => items.get(key) ?? null),
    setItem: vi.fn((key: string, value: string) => {
      items.set(key, value);
    }),
  };
}

async function freshStore(storage: unknown) {
  vi.resetModules();
  vi.stubGlobal("localStorage", storage);

  return await import("./ui.store");
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

describe("ui store: the title bar separator", () => {
  test("is off when nothing was saved", async () => {
    const store = await freshStore(fakeStorage());

    expect(store.uiStore.titlebarBorder).toBe(false);
  });

  test("is restored from a saved choice", async () => {
    const store = await freshStore(
      fakeStorage({ "dwall.ui.titlebar-border": "true" }),
    );

    expect(store.uiStore.titlebarBorder).toBe(true);
  });

  /**
   * Anything but the literals this module writes is treated as off: storage can hold a
   * value from another build, or something a user typed.
   */
  test("ignores a value it did not write", async () => {
    for (const value of ["1", "yes", "TRUE", ""]) {
      const store = await freshStore(
        fakeStorage({ "dwall.ui.titlebar-border": value }),
      );

      expect(store.uiStore.titlebarBorder).toBe(false);
    }
  });

  test("setting it updates the window and remembers the choice", async () => {
    const storage = fakeStorage();
    const store = await freshStore(storage);

    store.setTitlebarBorder(true);

    expect(store.uiStore.titlebarBorder).toBe(true);
    expect(storage.items.get("dwall.ui.titlebar-border")).toBe("true");

    store.setTitlebarBorder(false);

    expect(store.uiStore.titlebarBorder).toBe(false);
    expect(storage.items.get("dwall.ui.titlebar-border")).toBe("false");
  });

  /** A webview without storage still has to open and still has to toggle. */
  test("works when storage is unavailable", async () => {
    const store = await freshStore(undefined);

    expect(store.uiStore.titlebarBorder).toBe(false);

    store.setTitlebarBorder(true);

    expect(store.uiStore.titlebarBorder).toBe(true);
  });

  test("works when storage throws", async () => {
    const throwing = {
      getItem: () => {
        throw new Error("denied");
      },
      setItem: () => {
        throw new Error("denied");
      },
    };

    const store = await freshStore(throwing);

    expect(store.uiStore.titlebarBorder).toBe(false);

    store.setTitlebarBorder(true);

    expect(store.uiStore.titlebarBorder).toBe(true);
  });
});

describe("ui store: view switching", () => {
  test("switching to settings collapses the library drawer", async () => {
    const store = await freshStore(fakeStorage());

    store.setLibOpen(true);
    store.toggleView();

    expect(store.uiStore.view).toBe("settings");
    expect(store.uiStore.libOpen).toBe(false);
  });
});

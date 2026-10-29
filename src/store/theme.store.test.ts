import { beforeEach, describe, expect, test, vi } from "vitest";

function fakeStorage(getItem: () => string | null) {
  return { getItem: vi.fn(getItem), setItem: vi.fn() };
}

function fakeWindow(storage: unknown, dark = false) {
  return {
    localStorage: storage,
    matchMedia: vi.fn(() => ({
      matches: dark,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  };
}

async function freshStore(window: unknown) {
  vi.resetModules();
  vi.stubGlobal("window", window);

  return await import("./theme.store");
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

describe("theme store: storage can be unavailable", () => {
  test("falls back to the system appearance when reading storage throws", async () => {
    const store = await freshStore(
      fakeWindow(
        fakeStorage(() => {
          throw new Error("storage is disabled");
        }),
      ),
    );

    expect(store.themeStore.mode).toBe("system");
    expect(store.themeStore.resolved).toBe("light");
  });

  test("still restores a valid stored choice", async () => {
    const store = await freshStore(
      fakeWindow(
        fakeStorage(() => "dark"),
        true,
      ),
    );

    expect(store.themeStore.mode).toBe("dark");
    expect(store.themeStore.resolved).toBe("dark");
  });
});

import { beforeEach, describe, expect, test, vi } from "vitest";
import type { CatalogTheme } from "@/ipc";

const mocks = vi.hoisted(() => ({
  getThemeCatalog: vi.fn(),
  getMonitors: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  getThemeCatalog: mocks.getThemeCatalog,
  getMonitors: mocks.getMonitors,
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError, success: vi.fn(), info: vi.fn() },
}));

vi.mock("@/i18n", () => ({ t: (key: string) => key }));

const theme = (installed: boolean): CatalogTheme => ({
  id: "Big Sur",
  name: "Big Sur",
  source: "catalog",
  thumbnails: [],
  installed,
});

/** A fresh store, as a fresh window would see it. */
async function freshStore() {
  vi.resetModules();

  return await import("./catalog.store");
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.getMonitors.mockResolvedValue({});
});

describe("catalog store: reload races", () => {
  test("a reload keeps an install flag set while it was reading", async () => {
    mocks.getThemeCatalog.mockResolvedValueOnce([theme(false)]);

    const store = await freshStore();
    await store.load();
    expect(store.catalogStore.themes[0].installed).toBe(false);

    let resolveCatalog!: (themes: CatalogTheme[]) => void;
    mocks.getThemeCatalog.mockImplementationOnce(
      () =>
        new Promise<CatalogTheme[]>((resolve) => {
          resolveCatalog = resolve;
        }),
    );

    const reload = store.load();
    // The download finishes while the reload is in flight, and the catalog it is
    // waiting on was read before the theme was installed.
    store.markInstalled("Big Sur");
    resolveCatalog([theme(false)]);
    await reload;

    expect(store.catalogStore.themes[0].installed).toBe(true);
  });
});

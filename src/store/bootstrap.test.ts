import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Config } from "@/models/config";

const mocks = vi.hoisted(() => ({
  loadSettings: vi.fn(),
  loadCatalog: vi.fn(),
  settingsStore: { config: null as Config | null, status: "idle" as string },
  catalogStore: { status: "idle" as string },
  availableThemeIds: vi.fn((): string[] => []),
  syncFixed: vi.fn(async () => {}),
  syncRandom: vi.fn(),
  setMode: vi.fn(),
}));

vi.mock("./settings.store", () => ({
  load: mocks.loadSettings,
  settingsStore: mocks.settingsStore,
}));
vi.mock("./catalog.store", () => ({
  load: mocks.loadCatalog,
  catalogStore: mocks.catalogStore,
}));
vi.mock("./catalog.selectors", () => ({
  availableThemeIds: mocks.availableThemeIds,
}));
vi.mock("./fixed.store", () => ({ syncFromConfig: mocks.syncFixed }));
vi.mock("./random.store", () => ({ syncFromConfig: mocks.syncRandom }));
vi.mock("./mode.store", () => ({ setMode: mocks.setMode }));

const config = (mode: "fixed" | "random"): Config =>
  ({
    wallpaper_mode:
      mode === "random"
        ? { mode: "random", pool: ["Big Sur"] }
        : { mode: "fixed", monitor_specific_wallpapers: {} },
  }) as Config;

beforeEach(() => {
  vi.clearAllMocks();
  mocks.settingsStore.config = null;
  mocks.settingsStore.status = "idle";
  mocks.catalogStore.status = "idle";
  mocks.availableThemeIds.mockReturnValue([]);
});

describe("bootstrap", () => {
  test("loads both stores and derives the editing state from the config", async () => {
    mocks.settingsStore.config = config("random");
    mocks.settingsStore.status = "ready";
    mocks.availableThemeIds.mockReturnValue(["Alpha", "Beta"]);

    const { bootstrap } = await import("./bootstrap");
    await bootstrap();

    expect(mocks.loadSettings).toHaveBeenCalledTimes(1);
    expect(mocks.loadCatalog).toHaveBeenCalledTimes(1);
    expect(mocks.syncFixed).toHaveBeenCalledWith(mocks.settingsStore.config);
    expect(mocks.syncRandom).toHaveBeenCalledWith(mocks.settingsStore.config, [
      "Alpha",
      "Beta",
    ]);
    expect(mocks.setMode).toHaveBeenCalledWith("random");
  });

  test("a retry after a failed catalog read re-derives the pool from the config", async () => {
    const { bootstrap, reloadCatalog } = await import("./bootstrap");

    // First load: the catalog read fails, so the pool is seeded from an empty set.
    mocks.catalogStore.status = "error";
    await bootstrap();
    expect(mocks.syncRandom).toHaveBeenCalledTimes(1);

    // Reload succeeds with themes on disk: the same derivation has to run again.
    mocks.settingsStore.config = config("random");
    mocks.settingsStore.status = "ready";
    mocks.catalogStore.status = "ready";
    mocks.availableThemeIds.mockReturnValue(["Alpha", "Beta"]);

    await reloadCatalog();

    expect(mocks.syncRandom).toHaveBeenLastCalledWith(
      mocks.settingsStore.config,
      ["Alpha", "Beta"],
    );
    expect(mocks.syncFixed).toHaveBeenCalledTimes(2);
  });

  test("a failed reload does not derive state from a partial read", async () => {
    const { reloadCatalog, reloadSettings } = await import("./bootstrap");

    mocks.catalogStore.status = "error";
    await reloadCatalog();
    expect(mocks.syncFixed).not.toHaveBeenCalled();

    mocks.settingsStore.status = "error";
    await reloadSettings();
    expect(mocks.syncFixed).not.toHaveBeenCalled();
    expect(mocks.syncRandom).not.toHaveBeenCalled();
  });
});

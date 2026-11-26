import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Config } from "@/models/config";

const mocks = vi.hoisted(() => ({
  resolveMonitorThemes: vi.fn(),
  applyWallpaperMode: vi.fn(),
  settingsStore: {
    config: null as Config | null,
    saved: null as Config | null,
  },
}));

vi.mock("@/ipc", () => ({
  resolveMonitorThemes: mocks.resolveMonitorThemes,
}));
vi.mock("@/components/ui/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), info: vi.fn() },
}));
vi.mock("@/i18n", () => ({ t: (key: string) => key }));
vi.mock("./catalog.store", () => ({
  catalogStore: {
    monitors: [
      { device_path: "MON", friendly_name: "Display", position_index: 0 },
    ],
    themes: [],
  },
}));
vi.mock("./catalog.selectors", () => ({
  availableThemes: () => [
    { id: "Sunrise", name: "Sunrise", needsDownload: false },
  ],
  availableThemeIds: () => ["Sunrise"],
}));
vi.mock("./engine.store", () => ({ engineStore: { running: true } }));
vi.mock("./settings.store", () => ({
  applyWallpaperMode: mocks.applyWallpaperMode,
  settingsStore: mocks.settingsStore,
}));

const fixedConfig = (theme: string | null): Config =>
  ({
    wallpaper_mode: {
      mode: "fixed",
      monitor_specific_wallpapers: theme === null ? {} : theme,
    },
  }) as Config;

async function freshFixedStore() {
  vi.resetModules();

  return await import("./fixed.store");
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.settingsStore.config = fixedConfig("Sunrise");
  mocks.settingsStore.saved = fixedConfig("Sunrise");
  mocks.resolveMonitorThemes.mockImplementation(async (mode: unknown) => ({
    unified: true,
    themes: [
      { monitorId: "all", themeId: "Sunrise" },
      { monitorId: "MON", themeId: "Sunrise" },
    ],
    configured: ["all", "MON"],
    mode,
  }));
});

describe("fixed store: a stopped engine", () => {
  test("the stop notification clears the resolved mapping and the draft", async () => {
    const store = await freshFixedStore();
    await store.syncFromConfig(mocks.settingsStore.config);
    expect(store.isConfiguredTheme("all")).toBe(true);

    // A stop clears wallpaper_mode in the file; the settings store fires the
    // notification after mirroring that into its snapshot.
    const { syncStoppedSettings } = await import("./settings-refresh");
    syncStoppedSettings();

    expect(store.fixedStore.configThemes).toEqual({});
    expect(store.fixedStore.monitorThemes).toEqual({});
    expect(
      store.isConfiguredTheme("all"),
      "a cleared configuration is not a configured theme",
    ).toBe(false);
  });

  test("an empty mapping in the loaded config is not read as the stale resolved one", async () => {
    const store = await freshFixedStore();
    await store.syncFromConfig(mocks.settingsStore.config);
    expect(store.isConfiguredTheme("all")).toBe(true);

    // What the window sees after the stop command wrote the file, without the
    // in-memory stop notification (for example a reload that raced the command).
    mocks.settingsStore.config = fixedConfig(null);

    expect(store.isConfiguredTheme("all")).toBe(false);
    expect(store.isApplied("all")).toBe(false);
  });
});

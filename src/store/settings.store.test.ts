import { beforeEach, describe, expect, test, vi } from "vitest";
import type { Config } from "@/models/config";

// `vi.hoisted` because `vi.mock` factories are lifted above the imports: a plain
// `const` referenced from a factory is not initialised yet.
const mocks = vi.hoisted(() => ({
  readConfigFile: vi.fn(),
  writeConfigFile: vi.fn(),
  applyTheme: vi.fn(),
  moveDirectory: vi.fn(),
  refreshEngine: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  readConfigFile: mocks.readConfigFile,
  writeConfigFile: mocks.writeConfigFile,
  applyTheme: mocks.applyTheme,
  moveDirectory: mocks.moveDirectory,
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError, success: vi.fn(), info: vi.fn() },
}));

vi.mock("@/i18n", () => ({ t: (key: string) => key }));

vi.mock("./engine.store", () => ({ refresh: mocks.refreshEngine }));

const config = (overrides: Partial<Config> = {}): Config => ({
  network: null,
  title_bar_color_follows_windows_theme: false,
  image_format: "jpeg",
  position_source: {
    type: "AUTOMATIC",
    update_on_each_calculation: false,
    cache_minutes: 30,
  },
  auto_detect_color_scheme: true,
  lock_screen_wallpaper_enabled: true,
  themes_directory: "C:\\Users\\test\\AppData\\Roaming\\dwall\\themes",
  customized_themes_directory:
    "C:\\Users\\test\\AppData\\Roaming\\dwall\\customize",
  wallpaper_mode: { mode: "fixed", monitor_specific_wallpapers: {} },
  interval: 15,
  ...overrides,
});

/**
 * A fresh store, as a fresh window would see it.
 *
 * The store is a module-level singleton, so without this each test would inherit
 * the previous one's configuration.
 */
async function freshStore() {
  vi.resetModules();

  return await import("./settings.store");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("settings store: a configuration that is not loaded yet", () => {
  test("patch leaves a missing configuration missing instead of writing an empty one", async () => {
    const store = await freshStore();

    expect(store.settingsStore.config).toBeNull();

    store.patch({ interval: 30 });

    // Regression: an empty object here made `config.position_source.type` throw
    // inside the settings view's reactive tree, and there is no ErrorBoundary.
    expect(store.settingsStore.config).toBeNull();
  });

  test("patchPosition and the other draft writers are equally inert", async () => {
    const store = await freshStore();

    store.patchPosition({ latitude: 12.5 });
    store.setPositionType("MANUAL");
    store.patchSocks5({ host: "127.0.0.1" });

    expect(store.settingsStore.config).toBeNull();
  });

  test("save does nothing, so a stray click cannot persist an empty configuration", async () => {
    const store = await freshStore();

    await store.save();

    expect(mocks.writeConfigFile).not.toHaveBeenCalled();
  });
});

describe("settings store: loading", () => {
  test("a successful load fills the draft and the saved snapshot", async () => {
    const cfg = config({ interval: 60 });
    mocks.readConfigFile.mockResolvedValue(cfg);

    const store = await freshStore();
    await store.load();

    expect(store.settingsStore.status).toBe("ready");
    expect(store.settingsStore.error).toBeNull();
    expect(store.settingsStore.config).toEqual(cfg);
    expect(store.settingsStore.saved).toEqual(cfg);
  });

  test("a failed load is recorded rather than rendering as an empty form", async () => {
    mocks.readConfigFile.mockRejectedValue(new Error("invalid TOML"));

    const store = await freshStore();
    await store.load();

    expect(store.settingsStore.status).toBe("error");
    expect(store.settingsStore.error).toContain("invalid TOML");
    expect(store.settingsStore.config).toBeNull();
  });

  test("retrying after a failure recovers and clears the error", async () => {
    const cfg = config();
    mocks.readConfigFile
      .mockRejectedValueOnce(new Error("invalid TOML"))
      .mockResolvedValueOnce(cfg);

    const store = await freshStore();
    await store.load();
    expect(store.settingsStore.status).toBe("error");

    await store.load();

    expect(store.settingsStore.status).toBe("ready");
    expect(store.settingsStore.error).toBeNull();
    expect(store.settingsStore.config).toEqual(cfg);
  });
});

describe("settings store: the draft", () => {
  async function loadedStore(cfg: Config = config()) {
    mocks.readConfigFile.mockResolvedValue(cfg);

    const store = await freshStore();
    await store.load();

    return store;
  }

  test("patch merges into the loaded configuration", async () => {
    const store = await loadedStore();

    store.patch({ interval: 30 });

    expect(store.settingsStore.config?.interval).toBe(30);
    expect(store.settingsStore.config?.image_format).toBe("jpeg");
  });

  test("patchPosition replaces only the given coordinate", async () => {
    const store = await loadedStore(
      config({
        position_source: {
          type: "MANUAL",
          latitude: 1,
          longitude: 2,
          altitude: 3,
        },
      }),
    );

    store.patchPosition({ latitude: 45.5 });

    expect(store.settingsStore.config?.position_source).toEqual({
      type: "MANUAL",
      latitude: 45.5,
      longitude: 2,
      altitude: 3,
    });
  });

  test("switching the position source seeds the fields of the target variant", async () => {
    const store = await loadedStore();

    store.setPositionType("MANUAL");
    expect(store.settingsStore.config?.position_source).toEqual({
      type: "MANUAL",
      latitude: 0,
      longitude: 0,
      altitude: 875,
    });

    store.setPositionType("AUTOMATIC");
    expect(store.settingsStore.config?.position_source).toEqual({
      type: "AUTOMATIC",
      update_on_each_calculation: false,
      cache_minutes: 30,
    });
  });

  test("dirty follows the draft against the saved snapshot, and discard restores it", async () => {
    const store = await loadedStore();
    expect(store.isConfigDirty(store.settingsStore)).toBe(false);

    store.patch({ interval: 30 });
    expect(store.isConfigDirty(store.settingsStore)).toBe(true);

    store.discard();
    expect(store.isConfigDirty(store.settingsStore)).toBe(false);
    expect(store.settingsStore.config?.interval).toBe(15);
  });
});

describe("settings store: writing", () => {
  test("save persists the draft and moves the saved snapshot with it", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.writeConfigFile.mockResolvedValue(undefined);

    const store = await freshStore();
    await store.load();
    store.patch({ interval: 30 });
    await store.save();

    expect(mocks.writeConfigFile).toHaveBeenCalledTimes(1);
    expect(mocks.writeConfigFile.mock.calls[0][0].interval).toBe(30);
    expect(store.settingsStore.saving).toBe(false);
    expect(store.isConfigDirty(store.settingsStore)).toBe(false);
  });

  test("an edit made while the save is in flight stays dirty", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    let finishWrite!: () => void;
    mocks.writeConfigFile.mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finishWrite = resolve;
        }),
    );

    const store = await freshStore();
    await store.load();
    store.patch({ interval: 30 });

    const saving = store.save();
    // The write is still pending; this edit is not in the payload that was sent.
    store.patch({ interval: 60 });
    finishWrite();
    await saving;

    expect(store.settingsStore.saved?.interval).toBe(30);
    expect(store.isConfigDirty(store.settingsStore)).toBe(true);
  });

  test("a failed save keeps the draft dirty and surfaces the error", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.writeConfigFile.mockRejectedValue(new Error("read-only file"));

    const store = await freshStore();
    await store.load();
    store.patch({ interval: 30 });
    await store.save();

    expect(store.settingsStore.saveError).toContain("read-only file");
    expect(store.isConfigDirty(store.settingsStore)).toBe(true);
    expect(mocks.toastError).toHaveBeenCalled();
  });

  test("applying a wallpaper mode keeps the previous snapshot when the engine call fails", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.applyTheme.mockRejectedValue(new Error("daemon refused"));

    const store = await freshStore();
    await store.load();

    const mode = {
      mode: "random" as const,
      pool: ["Big Sur"],
    };

    await expect(store.applyWallpaperMode(mode)).rejects.toThrow(
      "daemon refused",
    );

    // The optimistic draft keeps the new mode, but nothing is marked as saved.
    expect(store.settingsStore.saved?.wallpaper_mode).toEqual({
      mode: "fixed",
      monitor_specific_wallpapers: {},
    });
    expect(mocks.toastError).toHaveBeenCalled();
  });

  /**
   * Applying a theme from the stage is not "save the settings form": a draft edit the
   * user has not committed must stay out of the file and keep the form dirty.
   */
  test("applying a mode does not commit unrelated unsaved edits", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.applyTheme.mockResolvedValue(undefined);

    const store = await freshStore();
    await store.load();

    store.patch({ interval: 30 });
    expect(store.isConfigDirty(store.settingsStore)).toBe(true);

    await store.applyWallpaperMode({ mode: "random", pool: ["Big Sur"] });

    const applied = mocks.applyTheme.mock.calls[0][0] as Config;
    expect(applied.interval).toBe(15);
    expect(applied.wallpaper_mode).toEqual({
      mode: "random",
      pool: ["Big Sur"],
    });

    expect(store.settingsStore.config?.interval).toBe(30);
    expect(store.isConfigDirty(store.settingsStore)).toBe(true);
  });
});

describe("settings store: moving the themes directory", () => {
  test("a successful move patches the directory and saves", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.moveDirectory.mockResolvedValue(undefined);
    mocks.writeConfigFile.mockResolvedValue(undefined);

    const store = await freshStore();
    await store.load();

    await store.changeThemesDirectory("D:\\themes");

    expect(mocks.moveDirectory).toHaveBeenCalledWith(
      "C:\\Users\\test\\AppData\\Roaming\\dwall\\themes",
      "D:\\themes",
    );
    expect(store.settingsStore.config?.themes_directory).toBe("D:\\themes");
    expect(mocks.writeConfigFile).toHaveBeenCalledTimes(1);
    expect(mocks.toastError).not.toHaveBeenCalled();
  });

  /**
   * A move can fail (a non-empty destination, a permission problem). The row has to
   * say so instead of leaving the old directory in place with only a log line.
   */
  test("a failed move is reported and nothing is written", async () => {
    mocks.readConfigFile.mockResolvedValue(config());
    mocks.moveDirectory.mockRejectedValue(
      new Error("destination is not empty"),
    );

    const store = await freshStore();
    await store.load();

    await store.changeThemesDirectory("D:\\themes");

    expect(mocks.toastError).toHaveBeenCalled();
    expect(store.settingsStore.config?.themes_directory).toBe(
      "C:\\Users\\test\\AppData\\Roaming\\dwall\\themes",
    );
    expect(mocks.writeConfigFile).not.toHaveBeenCalled();
  });
});

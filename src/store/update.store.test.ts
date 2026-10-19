import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  checkForUpdates: vi.fn(),
  relaunchApp: vi.fn(),
  toastInfo: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  checkForUpdates: mocks.checkForUpdates,
  relaunchApp: mocks.relaunchApp,
}));

vi.mock("@/components/ui/toast", () => ({
  toast: {
    info: mocks.toastInfo,
    error: mocks.toastError,
    success: vi.fn(),
  },
}));

vi.mock("@/i18n", () => ({
  t: (key: string, params?: Record<string, string>) =>
    params ? `${key}:${JSON.stringify(params)}` : key,
}));

vi.mock("./settings.store", () => ({
  settingsStore: { config: { network: { host: "127.0.0.1", port: 1086 } } },
}));

/** An update handle as the updater plugin hands it over. */
function updateHandle(version: string) {
  return {
    version,
    close: vi.fn().mockResolvedValue(undefined),
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
  };
}

/** A fresh store, as a freshly opened window would see it. */
async function freshStore() {
  vi.resetModules();

  return await import("./update.store");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("update store: checking on start-up", () => {
  test("an available update is announced once, with the version", async () => {
    mocks.checkForUpdates.mockResolvedValue(updateHandle("0.3.0"));

    const store = await freshStore();
    await store.checkOnStartup();
    await store.checkOnStartup();

    expect(store.updateStore.status).toBe("available");
    expect(store.updateStore.version).toBe("0.3.0");
    expect(mocks.checkForUpdates).toHaveBeenCalledTimes(1);
    expect(mocks.toastInfo).toHaveBeenCalledTimes(1);
    expect(mocks.toastInfo.mock.calls[0][0]).toContain("0.3.0");
    expect(mocks.toastInfo.mock.calls[0][1].action.label).toBeDefined();
  });

  test("being up to date is not announced", async () => {
    mocks.checkForUpdates.mockResolvedValue(null);

    const store = await freshStore();
    await store.checkOnStartup();

    expect(store.updateStore.status).toBe("latest");
    expect(mocks.toastInfo).not.toHaveBeenCalled();
  });

  /**
   * The check runs without being asked for, on a machine that may be offline: the
   * widget has to look untouched afterwards, not like a failed action the user never
   * started.
   */
  test("a start-up failure is silent and leaves the widget untouched", async () => {
    mocks.checkForUpdates.mockRejectedValue(new Error("offline"));

    const store = await freshStore();
    await store.checkOnStartup();

    expect(store.updateStore.status).toBe("idle");
    expect(store.updateStore.error).toBeNull();
    expect(mocks.toastInfo).not.toHaveBeenCalled();
    expect(mocks.toastError).not.toHaveBeenCalled();
  });

  /** The check has to honour the proxy the user configured, so it waits for settings. */
  test("the check uses the configured network", async () => {
    mocks.checkForUpdates.mockResolvedValue(null);

    const store = await freshStore();
    await store.checkOnStartup();

    expect(mocks.checkForUpdates).toHaveBeenCalledWith({
      host: "127.0.0.1",
      port: 1086,
    });
  });
});

describe("update store: checking on request", () => {
  test("a manual check reports a failure in the widget", async () => {
    mocks.checkForUpdates.mockRejectedValue(new Error("offline"));

    const store = await freshStore();
    await store.check();

    expect(store.updateStore.status).toBe("error");
    expect(store.updateStore.error).toContain("offline");
    expect(mocks.toastInfo).not.toHaveBeenCalled();
  });

  test("a manual check that finds an update updates the widget without a toast", async () => {
    mocks.checkForUpdates.mockResolvedValue(updateHandle("0.3.0"));

    const store = await freshStore();
    await store.check();

    expect(store.updateStore.status).toBe("available");
    expect(mocks.toastInfo).not.toHaveBeenCalled();
  });

  /**
   * A manual check after the start-up one has already run still has to work: the
   * "once per window" rule belongs to the automatic check, not to the widget's button.
   */
  test("a manual check still runs after the start-up one", async () => {
    mocks.checkForUpdates.mockResolvedValue(null);

    const store = await freshStore();
    await store.checkOnStartup();
    await store.check();

    expect(mocks.checkForUpdates).toHaveBeenCalledTimes(2);
  });

  /** The handle of the previous check is released, so the Rust resource table shrinks. */
  test("checking again releases the previous update handle", async () => {
    const first = updateHandle("0.3.0");
    mocks.checkForUpdates.mockResolvedValueOnce(first);
    mocks.checkForUpdates.mockResolvedValueOnce(null);

    const store = await freshStore();
    await store.check();
    await store.check();

    expect(first.close).toHaveBeenCalledTimes(1);
  });
});

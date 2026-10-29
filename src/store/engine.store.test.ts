import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getEngineStatus: vi.fn(),
  getDaemonLastError: vi.fn(),
  startEngine: vi.fn(),
  stopEngine: vi.fn(),
  reloadSettings: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  getEngineStatus: mocks.getEngineStatus,
  getDaemonLastError: mocks.getDaemonLastError,
  startEngine: mocks.startEngine,
  stopEngine: mocks.stopEngine,
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError, success: vi.fn(), info: vi.fn() },
}));

vi.mock("@/i18n", () => ({ t: (key: string) => key }));

vi.mock("./settings-refresh", () => ({
  reloadSettings: mocks.reloadSettings,
}));

/** A fresh store, as a fresh window would see it. */
async function freshStore() {
  vi.resetModules();

  return await import("./engine.store");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("engine store: status refresh", () => {
  test("a running daemon reports the error line from its log", async () => {
    mocks.getEngineStatus.mockResolvedValue(true);
    mocks.getDaemonLastError.mockResolvedValue(
      "ERROR [dwall] Every configured monitor failed",
    );

    const store = await freshStore();
    await store.refresh();

    expect(store.engineStore.running).toBe(true);
    expect(store.engineStore.lastError).toContain(
      "Every configured monitor failed",
    );
  });

  test("a running daemon with a clean log reports no error", async () => {
    mocks.getEngineStatus.mockResolvedValue(true);
    mocks.getDaemonLastError.mockResolvedValue(null);

    const store = await freshStore();
    await store.refresh();

    expect(store.engineStore.lastError).toBeNull();
  });

  /// The daemon keeps running after a failed cycle, so "running" alone would read
  /// as "all is well"; the reason has to survive a poll that only reads the status.
  test("a stopped daemon clears an error it reported before", async () => {
    mocks.getEngineStatus.mockResolvedValue(true);
    mocks.getDaemonLastError.mockResolvedValue(
      "ERROR [dwall] something failed",
    );

    const store = await freshStore();
    await store.refresh();
    expect(store.engineStore.lastError).not.toBeNull();

    // Stopped: there is no running daemon whose log could explain anything.
    mocks.getEngineStatus.mockResolvedValue(false);
    await store.refresh();

    expect(store.engineStore.running).toBe(false);
    expect(store.engineStore.lastError).toBeNull();
    expect(
      mocks.getDaemonLastError,
      "the log is only read while the daemon runs",
    ).toHaveBeenCalledTimes(1);
  });

  test("a failed status query is surfaced instead of silently reporting stopped", async () => {
    mocks.getEngineStatus.mockRejectedValue(new Error("ipc down"));

    const store = await freshStore();
    await store.refresh();

    expect(mocks.toastError).toHaveBeenCalled();
    expect(store.engineStore.running).toBe(false);
  });
});

describe("engine store: start and stop", () => {
  test("starting the engine marks it running", async () => {
    mocks.startEngine.mockResolvedValue(undefined);

    const store = await freshStore();
    await store.toggle();

    expect(mocks.startEngine).toHaveBeenCalledTimes(1);
    expect(store.engineStore.running).toBe(true);
  });

  test("stopping needs the second confirmation and clears the error", async () => {
    mocks.getEngineStatus.mockResolvedValue(true);
    mocks.getDaemonLastError.mockResolvedValue("ERROR [dwall] boom");
    mocks.stopEngine.mockResolvedValue(true);

    const store = await freshStore();
    await store.refresh();
    expect(store.engineStore.lastError).not.toBeNull();

    // First click only arms the confirmation.
    await store.toggle();
    expect(store.engineStore.pending).toBe(true);
    expect(mocks.stopEngine).not.toHaveBeenCalled();

    // Second click terminates.
    await store.toggle();
    expect(mocks.stopEngine).toHaveBeenCalledTimes(1);
    expect(mocks.reloadSettings).toHaveBeenCalledTimes(1);
    expect(store.engineStore.running).toBe(false);
    expect(store.engineStore.lastError).toBeNull();
    expect(store.engineStore.pending).toBe(false);
  });

  test("a failed stop leaves the engine running and clears the pending state", async () => {
    mocks.getEngineStatus.mockResolvedValue(true);
    mocks.getDaemonLastError.mockResolvedValue(null);
    mocks.stopEngine.mockRejectedValue(new Error("access denied"));

    const store = await freshStore();
    await store.refresh();
    await store.toggle();
    await store.toggle();

    expect(store.engineStore.running).toBe(true);
    expect(store.engineStore.pending).toBe(false);
    expect(mocks.toastError).toHaveBeenCalled();
  });
});

describe("engine store: poll hygiene", () => {
  test("a failure that keeps repeating is reported once, not on every tick", async () => {
    mocks.getEngineStatus.mockRejectedValue(new Error("ipc down"));

    const store = await freshStore();
    await store.refresh();
    await store.refresh();
    await store.refresh();

    expect(mocks.toastError).toHaveBeenCalledTimes(1);
  });

  test("an older refresh that resolves late does not overwrite a newer one", async () => {
    let resolveOlder!: (running: boolean) => void;
    mocks.getEngineStatus
      .mockImplementationOnce(
        () =>
          new Promise<boolean>((resolve) => {
            resolveOlder = resolve;
          }),
      )
      .mockResolvedValueOnce(false);

    const store = await freshStore();
    const older = store.refresh();
    const newer = store.refresh();
    await newer;
    expect(store.engineStore.running).toBe(false);

    resolveOlder(true);
    await older;

    expect(store.engineStore.running).toBe(false);
  });
});

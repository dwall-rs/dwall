import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  onThemeDownloadProgress: vi.fn(),
  downloadThemeAndExtract: vi.fn(),
  cancelThemeDownload: vi.fn(),
  markInstalled: vi.fn(),
  toastError: vi.fn(),
  logWarn: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  onThemeDownloadProgress: mocks.onThemeDownloadProgress,
  downloadThemeAndExtract: mocks.downloadThemeAndExtract,
  cancelThemeDownload: mocks.cancelThemeDownload,
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError, info: vi.fn(), success: vi.fn() },
}));

vi.mock("@/i18n", () => ({
  t: (key: string) => key,
}));

vi.mock("@/utils", () => ({
  logger: {
    child: () => ({
      warn: mocks.logWarn,
      error: vi.fn(),
      info: vi.fn(),
      debug: vi.fn(),
    }),
  },
}));

vi.mock("./catalog.store", () => ({
  markInstalled: mocks.markInstalled,
}));

vi.mock("./settings.store", () => ({
  settingsStore: { config: {} },
}));

interface Progress {
  taskId: string;
  downloadedBytes: number;
  totalBytes: number;
}

/** A fresh store, as a freshly opened window would see it. */
async function freshStore() {
  vi.resetModules();

  return await import("./download.store");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("download store: the progress listener", () => {
  /**
   * Registering the listener can fail (the event system is gone, the window is
   * closing). The store must not stay "downloading" forever, or every tile's
   * download button stays disabled with no way back.
   */
  test("a listener that fails to register does not wedge the store", async () => {
    mocks.onThemeDownloadProgress.mockRejectedValue(
      new Error("no event system"),
    );

    const store = await freshStore();
    const result = await store.downloadTheme("Big Sur");

    expect(result).toBe(false);
    expect(store.downloadStore.status).toBe("failed");
  });

  test("a successful download marks the theme installed and stops listening", async () => {
    const unlisten = vi.fn();
    mocks.onThemeDownloadProgress.mockResolvedValue(unlisten);
    mocks.downloadThemeAndExtract.mockResolvedValue(undefined);

    const store = await freshStore();
    const result = await store.downloadTheme("Big Sur");

    expect(result).toBe(true);
    expect(mocks.markInstalled).toHaveBeenCalledWith("Big Sur");
    expect(unlisten).toHaveBeenCalledTimes(1);
    expect(store.downloadStore.status).toBe("idle");
  });

  test("progress from another theme is ignored", async () => {
    const unlisten = vi.fn();
    let handler: ((progress: Progress) => void) | undefined;
    mocks.onThemeDownloadProgress.mockImplementation(async (h) => {
      handler = h;
      return unlisten;
    });
    // Keep the download pending so the captured handler can be exercised.
    mocks.downloadThemeAndExtract.mockImplementation(
      () => new Promise(() => {}),
    );

    const store = await freshStore();
    void store.downloadTheme("Big Sur");
    await Promise.resolve();

    handler?.({ taskId: "Catalina", downloadedBytes: 5, totalBytes: 10 });
    expect(store.downloadStore.percent).toBeNull();

    handler?.({ taskId: "Big Sur", downloadedBytes: 5, totalBytes: 10 });
    expect(store.downloadStore.percent).toBe(50);
  });
});

describe("download store: failures", () => {
  test("a failed download is reported and the listener is stopped", async () => {
    const unlisten = vi.fn();
    mocks.onThemeDownloadProgress.mockResolvedValue(unlisten);
    mocks.downloadThemeAndExtract.mockRejectedValue(new Error("disk full"));

    const store = await freshStore();
    const result = await store.downloadTheme("Big Sur");

    expect(result).toBe(false);
    expect(store.downloadStore.status).toBe("failed");
    expect(store.downloadStore.error).toContain("disk full");
    expect(mocks.toastError).toHaveBeenCalledTimes(1);
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  test("a cancellation returns to idle without a toast", async () => {
    mocks.onThemeDownloadProgress.mockResolvedValue(vi.fn());
    mocks.downloadThemeAndExtract.mockRejectedValue(
      new Error("Download cancelled"),
    );

    const store = await freshStore();
    const result = await store.downloadTheme("Big Sur");

    expect(result).toBe(false);
    expect(store.downloadStore.status).toBe("idle");
    expect(mocks.toastError).not.toHaveBeenCalled();
  });

  test("a cancel command that rejects is swallowed", async () => {
    let resolveDownload: (() => void) | undefined;
    mocks.onThemeDownloadProgress.mockResolvedValue(vi.fn());
    mocks.downloadThemeAndExtract.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveDownload = resolve;
        }),
    );
    mocks.cancelThemeDownload.mockRejectedValue(new Error("gone"));

    const store = await freshStore();
    const pending = store.downloadTheme("Big Sur");
    await Promise.resolve();

    await expect(store.cancelDownload()).resolves.toBeUndefined();
    expect(mocks.logWarn).toHaveBeenCalledTimes(1);

    resolveDownload?.();
    await pending;
  });
});

import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  applyWallpaperMode: vi.fn(),
}));

vi.mock("./settings.store", () => ({
  applyWallpaperMode: mocks.applyWallpaperMode,
}));

/** A fresh store holding `pool`, as a window that just loaded a random configuration. */
async function storeWithPool(pool: string[]) {
  vi.resetModules();
  const store = await import("./random.store");
  store.syncFromConfig(
    { wallpaper_mode: { mode: "random", pool } } as never,
    pool,
  );
  return store;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("random store: saving the pool", () => {
  /// The library stays clickable while the daemon restarts, so the store can change
  /// between the request and its answer. What was written is what the request carried,
  /// not what the store happens to hold when it returns.
  test("a pool change made while saving stays dirty", async () => {
    let release!: () => void;
    mocks.applyWallpaperMode.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          release = resolve;
        }),
    );

    const store = await storeWithPool(["Alpha", "Beta", "Gamma"]);
    store.toggle("Gamma");
    expect(store.isDirty(store.randomStore)).toBe(true);

    const saving = store.save();
    // Sent: Alpha, Beta. The user keeps working while the daemon restarts.
    store.toggle("Beta");

    release();
    await saving;

    expect(store.randomStore.saved).toEqual(["Alpha", "Beta"]);
    expect(store.randomStore.selected).toEqual(["Alpha"]);
    expect(
      store.isDirty(store.randomStore),
      "the change made during the save was never written",
    ).toBe(true);
  });

  test("a save that succeeds records what it sent", async () => {
    mocks.applyWallpaperMode.mockResolvedValue(undefined);

    const store = await storeWithPool(["Alpha", "Beta"]);
    store.toggle("Beta");

    await store.save();

    expect(mocks.applyWallpaperMode).toHaveBeenCalledWith({
      mode: "random",
      pool: ["Alpha"],
    });
    expect(store.randomStore.saved).toEqual(["Alpha"]);
    expect(store.randomStore.applied).toBe(true);
    expect(store.isDirty(store.randomStore)).toBe(false);
  });

  test("a failed save leaves the pool dirty and clears the saving flag", async () => {
    mocks.applyWallpaperMode.mockRejectedValue(
      new Error("daemon did not start"),
    );

    const store = await storeWithPool(["Alpha", "Beta"]);
    store.toggle("Beta");

    await store.save();

    expect(store.randomStore.saving).toBe(false);
    expect(store.isDirty(store.randomStore)).toBe(true);
  });
});

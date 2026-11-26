import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getColorSchemePause: vi.fn(),
  setColorSchemePause: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  getColorSchemePause: mocks.getColorSchemePause,
  setColorSchemePause: mocks.setColorSchemePause,
}));
vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError, success: vi.fn(), info: vi.fn() },
}));
vi.mock("@/i18n", () => ({ t: (key: string) => key }));
vi.mock("@/utils", () => ({
  logger: { child: () => ({ error: vi.fn() }) },
}));

const pause = (until: number) => ({ until, cause: "manual" as const });

async function freshStore() {
  vi.resetModules();

  return await import("./color-scheme-pause.store");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("colour-scheme pause store", () => {
  test("a successful refresh replaces the pause", async () => {
    mocks.getColorSchemePause.mockResolvedValue(pause(2_000));

    const store = await freshStore();
    await store.refreshColorSchemePause();

    expect(store.colorSchemePauseStore.pause()).toEqual(pause(2_000));
  });

  test("a failed refresh resolves and keeps the last known pause", async () => {
    mocks.getColorSchemePause.mockResolvedValueOnce(pause(2_000));

    const store = await freshStore();
    await store.refreshColorSchemePause();

    mocks.getColorSchemePause.mockRejectedValueOnce(new Error("ipc down"));
    await expect(store.refreshColorSchemePause()).resolves.toBeUndefined();

    expect(
      store.colorSchemePauseStore.pause(),
      "a poll failure must not pretend the pause is gone",
    ).toEqual(pause(2_000));
  });

  test("a failed pause reports it and keeps the previous value", async () => {
    mocks.getColorSchemePause.mockResolvedValueOnce(pause(2_000));

    const store = await freshStore();
    await store.refreshColorSchemePause();

    mocks.setColorSchemePause.mockRejectedValueOnce(
      new Error("read-only file"),
    );
    await expect(store.setColorSchemePauseFor(60)).resolves.toBe(false);

    expect(store.colorSchemePauseStore.pause()).toEqual(pause(2_000));
    expect(mocks.toastError).toHaveBeenCalledTimes(1);
  });

  test("a successful pause updates the value", async () => {
    mocks.setColorSchemePause.mockResolvedValue(pause(5_000));

    const store = await freshStore();
    const accepted = await store.setColorSchemePauseFor(60);

    expect(accepted).toBe(true);
    expect(store.colorSchemePauseStore.pause()).toEqual(pause(5_000));
  });
});

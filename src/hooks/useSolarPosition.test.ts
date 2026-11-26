import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  currentSolarPosition: vi.fn(),
}));

vi.mock("@/ipc", () => ({
  currentSolarPosition: mocks.currentSolarPosition,
}));
vi.mock("@/store/settings.store", () => ({
  settingsStore: { config: { position_source: { type: "AUTOMATIC" } } },
}));

const source = { type: "AUTOMATIC" as const, cache_minutes: 15 };

async function freshLoader() {
  vi.resetModules();

  return await import("./useSolarPosition");
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(0));
  mocks.currentSolarPosition.mockImplementation(async () => ({
    altitude: 12,
    azimuth: 34,
  }));
});

afterEach(() => {
  vi.useRealTimers();
});

describe("solar position polling", () => {
  test("simultaneous callers share one backend call", async () => {
    const { loadSolarPosition } = await freshLoader();

    const [a, b] = await Promise.all([
      loadSolarPosition(source),
      loadSolarPosition(source),
    ]);

    expect(a).toEqual(b);
    expect(mocks.currentSolarPosition).toHaveBeenCalledTimes(1);
  });

  test("a reading younger than the TTL is reused", async () => {
    const { loadSolarPosition } = await freshLoader();

    await loadSolarPosition(source);
    await loadSolarPosition(source);

    expect(mocks.currentSolarPosition).toHaveBeenCalledTimes(1);
  });

  test("the next poll after the TTL asks the backend again", async () => {
    const { loadSolarPosition } = await freshLoader();

    await loadSolarPosition(source);
    vi.advanceTimersByTime(30_001);
    await loadSolarPosition(source);

    expect(mocks.currentSolarPosition).toHaveBeenCalledTimes(2);
  });

  test("different position sources are not shared", async () => {
    const { loadSolarPosition } = await freshLoader();

    await loadSolarPosition(source);
    await loadSolarPosition({
      type: "MANUAL",
      latitude: 1,
      longitude: 2,
      altitude: 0,
    });

    expect(mocks.currentSolarPosition).toHaveBeenCalledTimes(2);
  });

  test("a failed read is retried instead of cached", async () => {
    const { loadSolarPosition } = await freshLoader();
    mocks.currentSolarPosition.mockRejectedValueOnce(new Error("denied"));

    await expect(loadSolarPosition(source)).rejects.toThrow("denied");
    await loadSolarPosition(source);

    expect(mocks.currentSolarPosition).toHaveBeenCalledTimes(2);
  });
});

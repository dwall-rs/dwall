import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createHoldableTimer } from "./holdable-timer";

describe("createHoldableTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("expires after the delay", () => {
    const expire = vi.fn();
    createHoldableTimer(expire).start(1_000);

    vi.advanceTimersByTime(999);
    expect(expire).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(expire).toHaveBeenCalledTimes(1);
  });

  it("keeps the time that was left when it is held", () => {
    const expire = vi.fn();
    const timer = createHoldableTimer(expire);

    timer.start(1_000);
    vi.advanceTimersByTime(400);
    timer.hold();

    // Held: however long the reader takes, nothing expires.
    vi.advanceTimersByTime(60_000);
    expect(expire).not.toHaveBeenCalled();

    timer.release();
    vi.advanceTimersByTime(599);
    expect(expire).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(expire).toHaveBeenCalledTimes(1);
  });

  it("stays held when a new delay arrives while held", () => {
    const expire = vi.fn();
    const timer = createHoldableTimer(expire);

    timer.start(1_000);
    timer.hold();
    // A toast updating its content must not resume a held countdown.
    timer.start(500);
    vi.advanceTimersByTime(10_000);
    expect(expire).not.toHaveBeenCalled();

    timer.release();
    vi.advanceTimersByTime(500);
    expect(expire).toHaveBeenCalledTimes(1);
  });

  it("does not restart a countdown that has already run out", () => {
    const expire = vi.fn();
    const timer = createHoldableTimer(expire);

    timer.start(100);
    vi.advanceTimersByTime(100);
    expect(expire).toHaveBeenCalledTimes(1);

    // Releasing after the deadline must not fire a second time.
    timer.release();
    vi.advanceTimersByTime(10_000);
    expect(expire).toHaveBeenCalledTimes(1);
  });

  it("stops for good", () => {
    const expire = vi.fn();
    const timer = createHoldableTimer(expire);

    timer.start(100);
    timer.stop();
    vi.advanceTimersByTime(10_000);
    expect(expire).not.toHaveBeenCalled();

    timer.release();
    vi.advanceTimersByTime(10_000);
    expect(expire).not.toHaveBeenCalled();
  });
});

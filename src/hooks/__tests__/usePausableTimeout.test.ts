import { describe, it, expect, vi, beforeEach } from "vitest";
import { usePausableTimeout } from "../usePausableTimeout";

describe("usePausableTimeout", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  it("should execute timeout callback normally", () => {
    const callback = vi.fn();
    const { start } = usePausableTimeout(callback, 1000);

    start();
    vi.advanceTimersByTime(1000);
    expect(callback).toHaveBeenCalled();
  });

  it("should pause and resume correctly", () => {
    const callback = vi.fn();
    const { start, pause, resume } = usePausableTimeout(callback, 1000);

    start();
    vi.advanceTimersByTime(500);
    pause();
    vi.advanceTimersByTime(1000);
    expect(callback).not.toHaveBeenCalled();

    resume();
    vi.advanceTimersByTime(500);
    expect(callback).toHaveBeenCalled();
  });

  it("remaining time should not be less than 0", () => {
    const callback = vi.fn();
    const { start, pause, resume } = usePausableTimeout(callback, 500);

    start();
    vi.advanceTimersByTime(300);
    pause();
    vi.advanceTimersByTime(300);
    resume();

    vi.advanceTimersByTime(200);
    expect(callback).toHaveBeenCalled();
  });

  it("should execute callback immediately when resuming with remaining time of 0", () => {
    const callback = vi.fn();
    const { start, pause, resume } = usePausableTimeout(callback, 500);

    start();
    vi.advanceTimersByTime(500);
    pause();

    resume();
    expect(callback).toHaveBeenCalled();
  });
});

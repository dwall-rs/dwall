import { describe, it, expect, vi, beforeEach } from "vitest";
import { usePausableTimeout } from "../usePausableTimeout";

describe("usePausableTimeout", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  it("应该正常执行超时回调", () => {
    const callback = vi.fn();
    const { start } = usePausableTimeout(callback, 1000);

    start();
    vi.advanceTimersByTime(1000);
    expect(callback).toHaveBeenCalled();
  });

  it("应该正确暂停和恢复", () => {
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

  it("剩余时间不应小于0", () => {
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

  it("恢复时剩余时间为0应立即执行回调", () => {
    const callback = vi.fn();
    const { start, pause, resume } = usePausableTimeout(callback, 500);

    start();
    vi.advanceTimersByTime(500);
    pause();

    resume();
    expect(callback).toHaveBeenCalled();
  });
});

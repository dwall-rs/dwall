import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  toastError: vi.fn(),
  logError: vi.fn(),
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: mocks.toastError },
}));
vi.mock("@/i18n", () => ({ t: (key: string) => key }));
vi.mock("@/utils", () => ({
  logger: { child: () => ({ error: mocks.logError }) },
}));

import { runExternalAction } from "./external-action";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("external actions", () => {
  test("a successful action is silent", async () => {
    const action = vi.fn().mockResolvedValue(undefined);

    await expect(
      runExternalAction("open the changelog", action),
    ).resolves.toBeUndefined();

    expect(action).toHaveBeenCalledTimes(1);
    expect(mocks.toastError).not.toHaveBeenCalled();
  });

  test("a failed action is reported and does not reject", async () => {
    const action = vi.fn().mockRejectedValue(new Error("no handler"));

    await expect(
      runExternalAction("open the changelog", action),
    ).resolves.toBeUndefined();

    expect(mocks.logError).toHaveBeenCalledTimes(1);
    expect(mocks.toastError).toHaveBeenCalledTimes(1);
  });
});

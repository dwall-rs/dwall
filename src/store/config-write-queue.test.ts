import { describe, expect, test } from "vitest";

import { queueConfigWrite } from "./config-write-queue";

describe("config write queue", () => {
  test("runs writes in the order they were requested", async () => {
    const order: string[] = [];
    let releaseFirst!: () => void;

    const first = queueConfigWrite(
      () =>
        new Promise<void>((resolve) => {
          releaseFirst = () => {
            order.push("first");
            resolve();
          };
        }),
    );
    const second = queueConfigWrite(async () => {
      order.push("second");
    });

    releaseFirst();
    await Promise.all([first, second]);

    expect(order).toEqual(["first", "second"]);
  });

  test("a rejected write does not stall the writes behind it", async () => {
    const order: string[] = [];

    const failing = queueConfigWrite(async () => {
      order.push("failing");
      throw new Error("write refused");
    });
    const after = queueConfigWrite(async () => {
      order.push("after");
    });

    await expect(failing).rejects.toThrow("write refused");
    await after;

    expect(order).toEqual(["failing", "after"]);
  });
});

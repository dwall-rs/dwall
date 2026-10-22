import { describe, expect, test } from "vitest";

import { resourceValue } from "./resource";

describe("resourceValue", () => {
  test("returns the value of a healthy resource", () => {
    const resource = Object.assign((): string | undefined => "value", {
      error: undefined,
    });

    expect(resourceValue(resource)).toBe("value");
  });

  test("returns undefined instead of throwing when the resource errored", () => {
    const resource = Object.assign(
      (): string | undefined => {
        throw new Error("boom");
      },
      { error: new Error("boom") },
    );

    expect(resourceValue(resource)).toBeUndefined();
  });
});

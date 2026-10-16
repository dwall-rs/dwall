import { describe, expect, test } from "vitest";

import { seedPool } from "./pool";

describe("seedPool", () => {
  test("drops a configured theme that is not on disk, so it can never be applied", () => {
    expect(seedPool(["Big Sur", "Catalina"], ["Big Sur"])).toEqual(["Big Sur"]);
  });

  test("drops a configured theme the catalog no longer lists", () => {
    expect(seedPool(["Gone", "Big Sur"], ["Big Sur"])).toEqual(["Big Sur"]);
  });

  test("keeps the configured order", () => {
    expect(seedPool(["Monterey", "Big Sur"], ["Big Sur", "Monterey"])).toEqual([
      "Monterey",
      "Big Sur",
    ]);
  });

  test("falls back to every available theme when no pool is configured", () => {
    expect(seedPool(null, ["Big Sur", "Catalina"])).toEqual([
      "Big Sur",
      "Catalina",
    ]);
  });

  test("is empty when nothing is installed, rather than offering what cannot apply", () => {
    expect(seedPool(["Catalina"], [])).toEqual([]);
  });
});

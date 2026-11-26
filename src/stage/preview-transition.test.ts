import { describe, expect, test } from "vitest";

import type { Wallpaper } from "@/models/types";
import { isSameDisplayedFrame } from "./preview-transition";

const frame = (index: number): Wallpaper =>
  ({ index, solar: { altitude: 0, azimuth: 0 } }) as Wallpaper;

describe("preview transition identity", () => {
  test("the same theme and frame does not start a transition", () => {
    expect(
      isSameDisplayedFrame(
        { themeId: "Big Sur", wallpaper: frame(5) },
        "Big Sur",
        frame(5),
      ),
    ).toBe(true);
  });

  test("a different frame of the same theme transitions", () => {
    expect(
      isSameDisplayedFrame(
        { themeId: "Big Sur", wallpaper: frame(5) },
        "Big Sur",
        frame(6),
      ),
    ).toBe(false);
  });

  test("the same frame index of a different theme transitions", () => {
    expect(
      isSameDisplayedFrame(
        { themeId: "Big Sur", wallpaper: frame(5) },
        "Catalina",
        frame(5),
      ),
    ).toBe(false);
  });

  test("a plate with nothing displayed does not compare equal", () => {
    expect(isSameDisplayedFrame(null, "Big Sur", frame(5))).toBe(false);
    expect(
      isSameDisplayedFrame(
        { themeId: "Big Sur", wallpaper: frame(5) },
        "Big Sur",
        null,
      ),
    ).toBe(false);
  });
});

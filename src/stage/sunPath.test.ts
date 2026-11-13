import { describe, expect, test } from "vitest";
import { splitAtAzimuthWrap, splitAtHorizon, xForAzimuth } from "./sunPath";

const at = (altitude: number) => ({ altitude });

describe("splitAtHorizon", () => {
  test("keeps an all-day path as one segment", () => {
    const segments = splitAtHorizon([at(2), at(15), at(4)]);

    expect(segments).toHaveLength(1);
    expect(segments[0].above).toBe(true);
    expect(segments[0].points.map((p) => p.altitude)).toEqual([2, 15, 4]);
  });

  test("splits at each horizon crossing and shares the interpolated crossing", () => {
    const segments = splitAtHorizon(
      [at(10), at(5), at(-5), at(-2), at(3)],
      () => at(0),
    );

    expect(segments.map((segment) => segment.above)).toEqual([
      true,
      false,
      true,
    ]);
    // The crossing sample is shared, so the strokes meet exactly at the horizon.
    expect(segments[0].points.map((p) => p.altitude)).toEqual([10, 5, 0]);
    expect(segments[1].points.map((p) => p.altitude)).toEqual([0, -5, -2, 0]);
    expect(segments[2].points.map((p) => p.altitude)).toEqual([0, 3]);
  });

  test("interpolates the azimuth at the horizon", () => {
    const segments = splitAtHorizon(
      [
        { altitude: 10, azimuth: 90 },
        { altitude: -10, azimuth: 270 },
      ],
      (before, after, ratio) => ({
        altitude: 0,
        azimuth: before.azimuth + (after.azimuth - before.azimuth) * ratio,
      }),
    );

    expect(segments).toHaveLength(2);
    expect(segments[0].points[1]).toEqual({ altitude: 0, azimuth: 180 });
    expect(segments[1].points[0]).toEqual({ altitude: 0, azimuth: 180 });
  });

  test("has nothing to split for an empty path", () => {
    expect(splitAtHorizon([])).toEqual([]);
  });
});
describe("xForAzimuth", () => {
  test("maps the compass linearly, not through a fold", () => {
    expect(xForAzimuth(0)).toBe(0);
    expect(xForAzimuth(90)).toBeCloseTo(25);
    expect(xForAzimuth(180)).toBeCloseTo(50);
    expect(xForAzimuth(270)).toBeCloseTo(75);
    expect(xForAzimuth(360)).toBeCloseTo(0);
  });

  test("normalises values outside the compass range", () => {
    expect(xForAzimuth(-90)).toBeCloseTo(75);
    expect(xForAzimuth(450)).toBeCloseTo(25);
  });
});
describe("splitAtAzimuthWrap", () => {
  test("breaks a path where north wraps from 360° to 0°", () => {
    const runs = splitAtAzimuthWrap([
      { azimuth: 350 },
      { azimuth: 355 },
      { azimuth: 2 },
      { azimuth: 10 },
    ]);

    expect(runs.map((run) => run.map((point) => point.azimuth))).toEqual([
      [350, 355],
      [2, 10],
    ]);
  });

  test("keeps an increasing path in one run", () => {
    const runs = splitAtAzimuthWrap([
      { azimuth: 10 },
      { azimuth: 90 },
      { azimuth: 200 },
    ]);

    expect(runs).toHaveLength(1);
  });
});

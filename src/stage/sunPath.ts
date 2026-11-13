// Responsibility: Pure geometry helpers for the sun path — azimuth → x coordinate, curve smoothing, and Y plotting math.

export type Point = readonly [number, number];

/**
 * Azimuth → x coordinate (0..100).
 *
 * A linear compass axis, not the old `50 - 44·sin(az)` fold. That fold put north and
 * south on the same x, so the 24-hour cycle came back on itself and read as an ellipse.
 * A linear axis gives the truthful shape: a daytime hump between the night tails, with
 * east at 25%, south at 50% and west at 75%.
 */
export const xForAzimuth = (azimuth: number): number =>
  ((((azimuth % 360) + 360) % 360) / 360) * 100;

/** Vertical plotting band (percent of the panel height). */
export const Y_TOP = 10;
export const Y_BOT = 84;

export interface AltitudeDomain {
  lo: number;
  hi: number;
}

/** Adaptive altitude domain covering every entry, with an 8% margin (minimum 4°). */
export function altitudeDomain(alts: readonly number[]): AltitudeDomain {
  const lo = Math.min(...alts);
  const hi = Math.max(...alts);
  const pad = Math.max((hi - lo) * 0.08, 4);
  return { lo: lo - pad, hi: hi + pad };
}

/** Altitude → y coordinate (percent) within [Y_TOP, Y_BOT]. */
export const yForAltitude = (alt: number, domain: AltitudeDomain): number =>
  Y_TOP + ((domain.hi - alt) / (domain.hi - domain.lo)) * (Y_BOT - Y_TOP);

/** Clamp an altitude's y into the plotting band so out-of-range markers stay visible and clickable. */
export const clampY = (alt: number, domain: AltitudeDomain): number =>
  Math.min(Math.max(yForAltitude(alt, domain), Y_TOP), Y_BOT);

/** Catmull-Rom → cubic Bézier: a smooth curve passing through every node (tension 0.5). */
export function smoothPath(points: readonly Point[]): string {
  if (points.length < 2) return "";
  if (points.length === 2) {
    return `M${points[0][0].toFixed(2)} ${points[0][1].toFixed(2)}L${points[1][0].toFixed(2)} ${points[1][1].toFixed(2)}`;
  }

  const parts = [`M${points[0][0].toFixed(2)} ${points[0][1].toFixed(2)}`];
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i - 1] ?? points[i];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[i + 2] ?? points[i + 1];

    const c1x = p1[0] + (p2[0] - p0[0]) / 6;
    const c1y = p1[1] + (p2[1] - p0[1]) / 6;
    const c2x = p2[0] - (p3[0] - p1[0]) / 6;
    const c2y = p2[1] - (p3[1] - p1[1]) / 6;

    parts.push(
      `C${c1x.toFixed(2)} ${c1y.toFixed(2)} ${c2x.toFixed(2)} ${c2y.toFixed(2)} ${p2[0].toFixed(2)} ${p2[1].toFixed(2)}`,
    );
  }
  return parts.join(" ");
}
/**
 * Split a sampled diurnal path into contiguous day and night segments.
 *
 * The full loop is correct for a 24-hour cycle, but drawing it as one solid line made
 * the night half look like the same kind of path as the daytime arc. Each crossing
 * point is kept in both neighbours, so the two strokes meet at the horizon.
 */
export interface HorizonSegment<T> {
  above: boolean;
  points: T[];
}

/** Build a sample at the horizon between two adjacent samples. */
export type HorizonInterpolator<T> = (before: T, after: T, ratio: number) => T;

export function splitAtHorizon<T extends { altitude: number }>(
  points: readonly T[],
  interpolate?: HorizonInterpolator<T>,
): HorizonSegment<T>[] {
  const segments: HorizonSegment<T>[] = [];
  let current: HorizonSegment<T> | undefined;

  for (const point of points) {
    const above = point.altitude >= 0;

    if (!current) {
      current = { above, points: [point] };
      continue;
    }

    if (current.above === above) {
      current.points.push(point);
      continue;
    }

    // Find the actual altitude-zero point and share it between both sides. Keeping the
    // raw sample would draw a few degrees of the daytime stroke below the horizon (and
    // the night stroke above it).
    const before = current.points[current.points.length - 1];
    const crossing = interpolate?.(
      before,
      point,
      before.altitude / (before.altitude - point.altitude),
    );

    if (crossing) {
      current.points.push(crossing);
      segments.push(current);
      current = { above, points: [crossing, point] };
    } else {
      current.points.push(point);
      segments.push(current);
      current = { above, points: [point] };
    }
  }

  if (current) segments.push(current);
  return segments;
}
/**
 * Split a chronological path where the azimuth wraps from 360° back to 0°.
 *
 * On a linear compass axis the two edges are the same direction (north); joining the
 * last sample of one edge to the first of the other would draw a chord straight across
 * the chart. The runs are separate strokes that meet at the edges instead.
 */
export function splitAtAzimuthWrap<T extends { azimuth: number }>(
  points: readonly T[],
): T[][] {
  const runs: T[][] = [];
  let current: T[] = [];

  for (const point of points) {
    const previous = current[current.length - 1];
    if (previous && Math.abs(point.azimuth - previous.azimuth) > 180) {
      if (current.length > 0) runs.push(current);
      current = [];
    }
    current.push(point);
  }

  if (current.length > 0) runs.push(current);
  return runs;
}

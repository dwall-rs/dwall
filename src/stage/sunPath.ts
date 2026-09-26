// 职责：太阳轨迹的纯几何工具——方位→横坐标与曲线平滑。

export type Point = readonly [number, number];

const DEG = Math.PI / 180;

/** 方位 → 横坐标（0..100）：东=6%、南=50%、西=94%。 */
export const xForAzimuth = (azimuth: number): number =>
  50 - 44 * Math.sin(azimuth * DEG);

/** Catmull-Rom → 三次贝塞尔：经过全部节点的平滑曲线（张力 0.5）。 */
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

/* ===== src/domain/solar.ts（仅 solarAt 改动） ===== */
// 职责：太阳位置纯计算。单一解析模型：高度角=正弦(全天)、方位角=线性 → 全程平滑、无分支折角。
import type { SolarPosition, Wallpaper } from "./types";

const r1 = (v: number) => Math.round(v * 10) / 10;

/** f=一天中的小时小数。6:00 日出、20:00 日落；正弦在 [6,20] 外自然为负（地平线以下），导数连续。 */
export function solarAt(f: number): SolarPosition {
  const t = (f - 6) / 14;
  return {
    altitude: r1(68 * Math.sin(Math.PI * t)),
    azimuth: r1(90 + 180 * t),
  };
}
export function estimateSolarPosition(d: Date = new Date()): SolarPosition {
  return solarAt(d.getHours() + d.getMinutes() / 60);
}

export function solarDistance(a: SolarPosition, b: SolarPosition): number {
  const dAlt = a.altitude - b.altitude;
  const dAz = Math.abs(a.azimuth - b.azimuth);
  return Math.hypot(dAlt * 1.2, dAz * 0.45);
}
export function nearestWallpaper(
  wallpapers: Wallpaper[],
  solar: SolarPosition,
): Wallpaper {
  let best = wallpapers[0],
    bd = Infinity;
  for (const w of wallpapers) {
    const d = solarDistance(w.solar, solar);
    if (d < bd) {
      bd = d;
      best = w;
    }
  }
  return best;
}

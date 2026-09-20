/* ===== src/domain/scene.ts ===== */
// 职责：由 (地貌, 太阳位置, 原生宽高) 纯函数地算出绘制参数；比例感知的地貌轮廓。不含 JSX。
import type { SceneType, SolarPosition } from "./types";

export const clamp = (v: number, a: number, b: number) =>
  Math.min(b, Math.max(a, v));
const hexToRgb = (hex: string): [number, number, number] => {
  const h = hex.replace("#", "");
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
  ];
};
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
export function lerpColor(c1: string, c2: string, t: number): string {
  const a = hexToRgb(c1),
    b = hexToRgb(c2),
    k = clamp(t, 0, 1);
  return `rgb(${Math.round(lerp(a[0], b[0], k))},${Math.round(lerp(a[1], b[1], k))},${Math.round(lerp(a[2], b[2], k))})`;
}

interface AltStop {
  alt: number;
  top: string;
  bottom: string;
  sun: string;
  warm: number;
}
const ALT_STOPS: AltStop[] = [
  { alt: -15, top: "#05070f", bottom: "#0d1230", sun: "#aab6e0", warm: 0 },
  { alt: -4, top: "#0d1230", bottom: "#3a2f55", sun: "#9aa6d8", warm: 0.2 },
  { alt: 2, top: "#574a6c", bottom: "#e0915a", sun: "#ffcf8a", warm: 1 },
  { alt: 10, top: "#6a7aa6", bottom: "#f0b070", sun: "#ffdba0", warm: 0.8 },
  { alt: 25, top: "#5a86b0", bottom: "#bcd6e6", sun: "#fff0c0", warm: 0.35 },
  { alt: 55, top: "#5f9fd0", bottom: "#d8ecff", sun: "#ffffff", warm: 0.08 },
  { alt: 90, top: "#4f93cc", bottom: "#cfe8ff", sun: "#ffffff", warm: 0 },
];
export function altitudeColors(alt: number) {
  const s = ALT_STOPS;
  if (alt <= s[0].alt) return s[0];
  for (let i = 0; i < s.length - 1; i++) {
    if (alt <= s[i + 1].alt) {
      const t = (alt - s[i].alt) / (s[i + 1].alt - s[i].alt);
      return {
        top: lerpColor(s[i].top, s[i + 1].top, t),
        bottom: lerpColor(s[i].bottom, s[i + 1].bottom, t),
        sun: lerpColor(s[i].sun, s[i + 1].sun, t),
        warm: lerp(s[i].warm, s[i + 1].warm, t),
      };
    }
  }
  return s[s.length - 1];
}

const GROUND_BASE: Partial<Record<SceneType, string>> = {
  island: "#2c3a2a",
  peak: "#26303f",
  lake: "#16314a",
  dunes: "#8a5a2c",
  ridge: "#1d2738",
  coast: "#13314a",
};

export interface SceneSpec {
  skyTop: string;
  skyBottom: string;
  sun: string;
  sunX: number;
  sunY: number;
  sunR: number;
  glowR: number;
  warm: number;
  night: boolean;
  horizonY: number;
  groundFill: string;
  daylight: number;
}
export function getSceneSpec(
  type: SceneType,
  solar: SolarPosition,
  w: number,
  h: number,
): SceneSpec {
  const alt = clamp(solar.altitude, -15, 90);
  const col = altitudeColors(alt);
  const night = alt < -2;
  const horizonY = h * 0.68;
  const sunY = horizonY - (alt / 90) * (horizonY - h * 0.12);
  const az = clamp(solar.azimuth, 70, 290);
  const sunX = w * 0.08 + ((az - 70) / 220) * (w * 0.84);
  const sunR = Math.min(w, h) * 0.045;
  const glowR = Math.min(w, h) * (0.1 + col.warm * 0.14);
  const daylight = clamp((alt + 6) / 26, 0, 1);
  const groundFill = lerpColor(
    "#0a0c14",
    GROUND_BASE[type] ?? "#1a1c26",
    daylight,
  );
  return {
    skyTop: col.top,
    skyBottom: col.bottom,
    sun: col.sun,
    sunX,
    sunY,
    sunR,
    glowR,
    warm: col.warm,
    night,
    horizonY,
    groundFill,
    daylight,
  };
}

/** 比例感知的地貌轮廓：按实际 w/h 生成，不锁死 16/10。 */
export function groundPath(type: SceneType, w: number, h: number): string {
  switch (type) {
    case "peak":
      return `M0 ${h} L${w * 0.16} ${h * 0.3} L${w * 0.3} ${h * 0.52} L${w * 0.5} ${h * 0.18} L${w * 0.68} ${h * 0.5} L${w * 0.86} ${h * 0.3} L${w} ${h * 0.46} L${w} ${h} Z`;
    case "ridge":
      return `M0 ${h} L${w * 0.1} ${h * 0.52} L${w * 0.22} ${h * 0.34} L${w * 0.36} ${h * 0.56} L${w * 0.5} ${h * 0.28} L${w * 0.66} ${h * 0.54} L${w * 0.8} ${h * 0.36} L${w} ${h * 0.58} L${w} ${h} Z`;
    case "island":
      return `M${w * 0.28} ${h * 0.66} Q${w * 0.4} ${h * 0.42} ${w * 0.52} ${h * 0.52} Q${w * 0.66} ${h * 0.4} ${w * 0.74} ${h * 0.66} Q${w * 0.78} ${h * 0.7} ${w * 0.72} ${h * 0.72} L${w * 0.3} ${h * 0.72} Q${w * 0.24} ${h * 0.7} ${w * 0.28} ${h * 0.66} Z`;
    case "dunes":
      return `M0 ${h} L0 ${h * 0.7} Q${w * 0.25} ${h * 0.58} ${w * 0.5} ${h * 0.68} T${w} ${h * 0.64} L${w} ${h} Z`;
    case "coast":
      return `M${w * 0.62} ${h * 0.56} Q${w * 0.76} ${h * 0.46} ${w * 0.92} ${h * 0.58} L${w * 0.92} ${h * 0.66} Q${w * 0.76} ${h * 0.56} ${w * 0.62} ${h * 0.64} Z`;
    case "lake":
      return `M0 ${h * 0.6} L${w * 0.14} ${h * 0.4} L${w * 0.3} ${h * 0.56} L${w * 0.48} ${h * 0.36} L${w * 0.66} ${h * 0.54} L${w * 0.84} ${h * 0.42} L${w} ${h * 0.52} L${w} ${h * 0.6} Z`;
    default:
      return "";
  }
}
export const hasWater = (type: SceneType): boolean =>
  type === "island" || type === "lake" || type === "coast";

// 职责：把 (地貌, 时段) 渲染成自包含 SVG；纯 props，零 store，零副作用。
import { getSceneSpec, groundPath, hasWater, lerpColor } from "@/domain/scene";
import type { SceneType, SolarPosition } from "@/domain/types";
import { clsx } from "@/utils";
import { createMemo, createUniqueId } from "solid-js";

interface SceneProps {
  type: SceneType;
  solar: SolarPosition;
  w: number;
  h: number;
  fit?: "cover" | "contain";
  class?: string;
}

export function Scene(props: SceneProps) {
  const uid = createUniqueId();

  const s = createMemo(() =>
    getSceneSpec(props.type, props.solar, props.w, props.h),
  );
  const par = props.fit === "cover" ? "xMidYMid slice" : "xMidYMid meet";

  const stars = s().night
    ? [
        [0.15, 0.16],
        [0.8, 0.22],
        [0.62, 0.12],
        [0.35, 0.28],
        [0.9, 0.48],
        [0.22, 0.38],
      ].map(([fx, fy], i) => (
        <circle
          cx={fx * props.w}
          cy={fy * props.h}
          r={
            i % 2
              ? Math.min(props.w, props.h) * 0.004
              : Math.min(props.w, props.h) * 0.003
          }
          fill="#fff"
          opacity={0.9}
        />
      ))
    : null;

  return (
    <svg
      viewBox={`0 0 ${props.w} ${props.h}`}
      preserveAspectRatio={par}
      class={clsx("block h-full w-full", props.class)}
      aria-label={`${props.type} scene`}
    >
      <defs>
        <linearGradient id={`sky${uid}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color={s().skyTop} />
          <stop offset="1" stop-color={s().skyBottom} />
        </linearGradient>
        <radialGradient id={`glow${uid}`} cx="0.5" cy="0.5" r="0.5">
          <stop offset="0" stop-color={s().sun} stop-opacity="0.85" />
          <stop offset="1" stop-color={s().sun} stop-opacity="0" />
        </radialGradient>
        {props.type === "earth" && (
          <radialGradient id={`pl${uid}`} cx="0.4" cy="0.4" r="0.8">
            <stop offset="0" stop-color="#2f7fd0" />
            <stop offset="1" stop-color="#0a1830" />
          </radialGradient>
        )}
      </defs>
      <rect width={props.w} height={props.h} fill={`url(#sky${uid})`} />
      {stars}
      {s().warm > 0.04 && (
        <circle
          cx={s().sunX}
          cy={s().sunY}
          r={s().glowR}
          fill={`url(#glow${uid})`}
        />
      )}
      <circle
        cx={s().sunX}
        cy={s().sunY}
        r={s().sunR}
        fill={s().sun}
        opacity={s().night ? 0.85 : 0.95}
      />
      {hasWater(props.type) && (
        <>
          <rect
            x={0}
            y={s().horizonY}
            width={props.w}
            height={props.h - s().horizonY}
            fill={lerpColor(s().skyBottom, "#0a1626", 0.45)}
          />
          <ellipse
            cx={s().sunX}
            cy={s().horizonY + (props.h - s().horizonY) * 0.35}
            rx={props.w * 0.05}
            ry={(props.h - s().horizonY) * 0.3}
            fill={s().sun}
            opacity={0.1 * s().daylight + 0.04}
          />
        </>
      )}
      {props.type === "earth" ? (
        <>
          <circle
            cx={props.w * 0.75}
            cy={props.h * 1.28}
            r={Math.min(props.w, props.h) * 0.8}
            fill={`url(#pl${uid})`}
          />
          <circle
            cx={props.w * 0.75}
            cy={props.h * 1.28}
            r={Math.min(props.w, props.h) * 0.8}
            fill="none"
            stroke="#7fd0ff"
            stroke-width={Math.min(props.w, props.h) * 0.008}
            opacity={0.35}
          />
        </>
      ) : (
        <path
          d={groundPath(props.type, props.w, props.h)}
          fill={s().groundFill}
        />
      )}
    </svg>
  );
}

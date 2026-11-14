// Responsibility: the signature sun-path instrument — the real daily arc for the
// observer, with the theme's frames placed on it and the live sun on top. It is dark
// in both appearances on purpose: it is an instrument, not another card.
import type { SolarPosition, Wallpaper } from "@/models/types";
import { getSolarPath } from "@/ipc";
import { resourceValue } from "@/lib/resource";
import { t } from "@/i18n";
import { settingsStore } from "@/store/settings.store";
import { clsx, formatAngle } from "@/utils";
import { createMemo, createResource, createSignal, For, Show } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  type AltitudeDomain,
  Y_BOT,
  altitudeDomain,
  clampY,
  type Point,
  smoothPath,
  splitAtAzimuthWrap,
  splitAtHorizon,
  xForAzimuth,
  yForAltitude,
} from "./sunPath";

interface Props {
  wallpapers: Wallpaper[];
  current: SolarPosition;
  /** Position of the matched solar-angle entry within the array (not the image index) */
  matchedEntry: number | null;
  /** Entry the user picked; omit it, and `onSelect`, for a track that only reports. */
  selIndex?: number | null;
  onSelect?: (index: number | null) => void;
  fullWidth?: boolean;
}

/** Interpolate a heading across the 360°→0° seam. */
const azimuthAtHorizon = (before: number, after: number, ratio: number) => {
  const delta = ((after - before + 540) % 360) - 180;
  return (before + delta * ratio + 360) % 360;
};

export function SunPathPanel(props: Props) {
  const [hover, setHover] = createSignal<number | null>(null);
  const [hoverSun, setHoverSun] = createSignal(false);
  const interactive = () => props.onSelect !== undefined;

  const [path] = createResource(
    () => settingsStore.config?.position_source,
    (ps) => getSolarPath(ps, Math.floor(Date.now() / 1000)),
  );
  // Keep all 24 hours: the sun has to stay on the track after it sets, and the
  // below-horizon theme frames need somewhere to sit. The split below is what keeps
  // the loop from reading as one solid orbit.
  const pathPoints = createMemo(() => resourceValue(path) ?? []);

  const domain = createMemo<AltitudeDomain>(() => {
    const pathAlts = pathPoints().map((p) => p.altitude);
    return altitudeDomain([
      props.current.altitude,
      0,
      ...pathAlts,
      ...(pathAlts.length === 0
        ? props.wallpapers.map((w) => w.solar.altitude)
        : []),
    ]);
  });
  const Yp = (alt: number) => yForAltitude(alt, domain());
  const hor = () => Yp(0);

  const cx = createMemo(() => xForAzimuth(props.current.azimuth));
  const cy = createMemo(() => Yp(props.current.altitude));

  const matched = createMemo(() =>
    props.matchedEntry != null
      ? props.wallpapers[props.matchedEntry]
      : undefined,
  );

  const segments = createMemo(() =>
    splitAtAzimuthWrap(pathPoints()).flatMap((run) =>
      splitAtHorizon(run, (before, after, ratio) => ({
        altitude: 0,
        azimuth: azimuthAtHorizon(before.azimuth, after.azimuth, ratio),
      })).map((segment) => ({
        above: segment.above,
        d: smoothPath(
          segment.points.map(
            (p): Point => [xForAzimuth(p.azimuth), Yp(p.altitude)],
          ),
        ),
      })),
    ),
  );

  return (
    <section
      aria-label={t("stage.sunPath")}
      class={clsx(
        "relative overflow-hidden rounded-[18px] border border-white/10 bg-[#0b121b] p-4 text-white shadow-[0_18px_40px_-24px_rgba(0,0,0,.65)]",
        props.fullWidth ? "w-full" : "w-full",
      )}
    >
      {/* A faint dawn wash above the horizon; the grid and the arc carry the data. */}
      <div class="pointer-events-none absolute inset-0 bg-[radial-gradient(120%_75%_at_72%_-10%,rgba(236,164,66,.16),transparent_55%)]" />

      <div class="relative flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="text-[10.5px] font-medium tracking-[0.02em] text-white/45">
            {t("stage.sunPath")}
          </div>
          <div class="mt-1 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 font-mono">
            <span class="text-[22px] leading-none text-white">
              {formatAngle(props.current.altitude)}°
            </span>
            <span class="text-[10px] text-white/45">
              {t("app.engine.altitude")}
            </span>
            <span class="text-[22px] leading-none text-white">
              {formatAngle(props.current.azimuth)}°
            </span>
            <span class="text-[10px] text-white/45">
              {t("app.engine.azimuth")}
            </span>
          </div>
        </div>
        <span class="flex shrink-0 items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-2 py-1 font-mono text-[9.5px] text-white/65">
          <span
            class={clsx(
              "size-1.5 rounded-full",
              props.current.altitude < 0
                ? "bg-white/45"
                : "bg-solar shadow-[0_0_7px_var(--solar)]",
            )}
          />
          {props.current.altitude < 0 ? "☾" : "☀"}
        </span>
      </div>

      <div
        class={clsx("relative mt-3 w-full", props.fullWidth ? "h-52" : "h-42")}
      >
        <svg
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          class="absolute inset-0 h-full w-full"
          aria-hidden="true"
        >
          <defs>
            <linearGradient id="spArc" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0" stop-color="#d98d2f" />
              <stop offset=".5" stop-color="#f2bd63" />
              <stop offset="1" stop-color="#d98d2f" />
            </linearGradient>
            <linearGradient id="spGround" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="#ffffff" stop-opacity=".07" />
              <stop offset="1" stop-color="#ffffff" stop-opacity=".01" />
            </linearGradient>
          </defs>

          {/* A 10° grid, so the arc is readable rather than decorative. */}
          <For each={[20, 40, 60]}>
            {(y) => (
              <line
                x1="0"
                x2="100"
                y1={y}
                y2={y}
                stroke="#ffffff"
                stroke-opacity=".055"
                vector-effect="non-scaling-stroke"
              />
            )}
          </For>

          {/* The daytime arc is the path; the night half is a dashed continuation.
              Feeding all 24h samples keeps the current position and the below-horizon
              frames on the line instead of leaving them floating. */}
          <For each={segments()}>
            {(segment) => (
              <Show when={segment.d}>
                <path
                  d={segment.d}
                  fill="none"
                  stroke={segment.above ? "url(#spArc)" : "#f2bd63"}
                  stroke-width={segment.above ? 1.6 : 1.1}
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-dasharray={segment.above ? undefined : "3 4"}
                  opacity={segment.above ? 0.9 : 0.38}
                  vector-effect="non-scaling-stroke"
                />
              </Show>
            )}
          </For>

          <rect
            x="0"
            y={hor()}
            width="100"
            height={Math.max(0, 100 - hor())}
            fill="url(#spGround)"
          />
          <line
            x1="0"
            x2="100"
            y1={hor()}
            y2={hor()}
            stroke="#ffffff"
            stroke-opacity=".22"
            vector-effect="non-scaling-stroke"
          />
          <Show when={matched()}>
            {(m) => (
              <line
                x1={cx()}
                y1={cy()}
                x2={xForAzimuth(m().solar.azimuth)}
                y2={clampY(m().solar.altitude, domain())}
                stroke="#f2bd63"
                stroke-opacity=".5"
                stroke-dasharray="2 4"
                vector-effect="non-scaling-stroke"
              />
            )}
          </Show>
        </svg>

        <For each={props.wallpapers}>
          {(wp, i) => {
            const sel = createMemo(() => wp.index === props.selIndex);
            const mat = createMemo(() => i() === props.matchedEntry);
            const hov = createMemo(() => i() === hover());
            return (
              <Dynamic
                component={interactive() ? "button" : "span"}
                {...(interactive() ? { type: "button" as const } : {})}
                title={`${t("stage.altitude")} ${formatAngle(wp.solar.altitude)}° · ${t("stage.azimuth")} ${formatAngle(wp.solar.azimuth)}°`}
                onClick={
                  props.onSelect ? () => props.onSelect?.(wp.index) : undefined
                }
                onMouseEnter={() => setHover(i())}
                onMouseLeave={() => setHover(null)}
                style={{
                  left: `${xForAzimuth(wp.solar.azimuth)}%`,
                  top: `${clampY(wp.solar.altitude, domain())}%`,
                }}
                class={clsx(
                  "absolute -translate-x-1/2 -translate-y-1/2 rounded-full border border-[#0b121b] transition-all outline-none",
                  "before:absolute before:-inset-2.5 before:content-['']",
                  "focus-visible:ring-2 focus-visible:ring-solar focus-visible:ring-offset-2 focus-visible:ring-offset-[#0b121b]",
                  mat() || sel()
                    ? "size-2.5 bg-solar shadow-[0_0_0_2px_#0b121b,0_0_0_3px_rgba(242,189,99,.8)]"
                    : hov()
                      ? "size-2.5 bg-white"
                      : "size-1.5 bg-white/50",
                )}
              />
            );
          }}
        </For>

        <div
          style={{ left: `${cx()}%`, top: `${cy()}%` }}
          class="absolute flex size-6 -translate-x-1/2 -translate-y-1/2 items-center justify-center"
          onMouseEnter={() => setHoverSun(true)}
          onMouseLeave={() => setHoverSun(false)}
          title={`${t("stage.altitude")} ${formatAngle(props.current.altitude)}° · ${t("stage.azimuth")} ${formatAngle(props.current.azimuth)}°`}
        >
          <span
            class={clsx(
              "size-2.5 rounded-full transition-transform",
              hoverSun() && "scale-125",
              props.current.altitude < 0
                ? "bg-white/55 ring-2 ring-white/15"
                : "bg-solar shadow-[0_0_12px_2px_rgba(242,189,99,.65)] ring-2 ring-[#0b121b]",
            )}
          />
        </div>

        <span
          class="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 whitespace-nowrap rounded bg-[#0b121b]/80 px-1.5 py-0.5 font-mono text-[8.5px] text-white/40"
          style={{ top: `${hor()}%` }}
        >
          {t("stage.horizon")}
        </span>
        <span
          class="absolute -left-0.5 -translate-y-1/2 font-mono text-[8.5px] text-white/35"
          style={{ top: `${Y_BOT}%` }}
        >
          {(Math.round(domain().lo) || 0).toFixed(0)}°
        </span>

        <For
          each={[
            { az: 90, label: t("stage.east") },
            { az: 180, label: t("stage.south") },
            { az: 270, label: t("stage.west") },
          ]}
        >
          {(mark) => (
            <span
              class="absolute bottom-0 -translate-x-1/2 font-mono text-[9px] text-white/35"
              style={{ left: `${xForAzimuth(mark.az)}%` }}
            >
              {mark.label}
            </span>
          )}
        </For>
      </div>

      <Show when={hoverSun()}>
        <span class="pointer-events-none absolute right-3 top-3 rounded-md border border-white/10 bg-[#111b27]/95 px-2 py-1 font-mono text-[10px] text-white/80 shadow-lg backdrop-blur">
          {props.current.altitude < 0 ? "☾" : "☀"}{" "}
          {props.current.altitude.toFixed(1)}° ·{" "}
          {props.current.azimuth.toFixed(1)}°
        </span>
      </Show>
    </section>
  );
}

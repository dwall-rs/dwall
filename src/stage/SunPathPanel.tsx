// 职责：装饰性日轨丝带——真椭圆；底部预留标签条带防重叠；地平线标注置于中段空位。
import type { SolarPosition, Wallpaper } from "@/domain/types";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { createMemo, createSignal, For, Show } from "solid-js";
import { themeStore } from "~/store/theme.store";

// 垂直轴：单一线性比例；Y_BOT=84 给底部标签留出条带，地下最深 −20°
const ALT_MAX = 90,
  ALT_MIN = -20,
  Y_TOP = 10,
  Y_BOT = 84;
const Yp = (alt: number) =>
  Y_TOP + ((ALT_MAX - alt) / (ALT_MAX - ALT_MIN)) * (Y_BOT - Y_TOP);
const HOR = Yp(0);
const Xaz = (az: number) => 50 - 44 * Math.cos((Math.PI * (az - 90)) / 180);
const altAt = (az: number) => 68 * Math.sin((Math.PI * (az - 90)) / 180);
const band = (alt: number) =>
  alt < 0 ? "bg-muted-foreground" : alt < 20 ? "bg-(--warning)" : "bg-primary";

interface Props {
  wallpapers: Wallpaper[];
  current: SolarPosition;
  /** 匹配到的太阳角条目在数组中的位置（非图片序号） */
  matchedEntry: number | null;
  selIndex: number | null;
  onSelect: (index: number | null) => void;
}

export function SunPathPanel(props: Props) {
  const [hover, setHover] = createSignal<number | null>(null);

  const cx = createMemo(() => Xaz(props.current.azimuth)),
    cy = createMemo(() => Yp(props.current.altitude));

  const matched = createMemo(() =>
    props.matchedEntry != null
      ? props.wallpapers[props.matchedEntry]
      : undefined,
  );

  const path = createMemo(() => {
    const pts: string[] = [];
    for (let az = 64; az <= 296; az += 2) {
      const alt = altAt(az);
      if (alt < ALT_MIN) continue;
      pts.push(
        `${pts.length ? "L" : "M"}${Xaz(az).toFixed(2)} ${Yp(alt).toFixed(2)}`,
      );
    }
    return pts.join(" ");
  });

  return (
    <div class="mx-auto w-full max-w-155 shrink-0">
      <div class="relative h-28 shrink-0 overflow-hidden rounded-xl border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_6px_18px_rgba(0,0,0,.06)]">
        <div
          class={clsx(
            "pointer-events-none absolute inset-x-0 top-0",
            themeStore.resolved === "light" &&
              "bg-[radial-gradient(120%_80%_at_50%_-10%,color-mix(in_oklch,var(--color-sky-500),transparent_5%),transparent_60%)]",
          )}
          style={{ height: `${HOR}%` }}
        />

        <svg
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          class="absolute inset-0 h-full w-full"
          aria-label={t("stage.sunPath")}
        >
          <defs>
            <linearGradient id="spArc" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0" stop-color="var(--warning)" />
              <stop offset=".5" stop-color="var(--success)" />
              <stop offset="1" stop-color="var(--warning)" />
            </linearGradient>
            <linearGradient id="spGround" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="0"
                stop-color="var(--muted-foreground)"
                stop-opacity=".16"
              />
              <stop
                offset="1"
                stop-color="var(--muted-foreground)"
                stop-opacity=".04"
              />
            </linearGradient>
          </defs>

          <path
            d={path()}
            fill="none"
            stroke="url(#spArc)"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            opacity=".8"
            vector-effect="non-scaling-stroke"
          />
          <rect
            x="0"
            y={HOR}
            width="100"
            height={100 - HOR}
            fill="var(--card)"
            opacity=".45"
          />
          <rect
            x="0"
            y={HOR}
            width="100"
            height={100 - HOR}
            fill="url(#spGround)"
          />
          <line
            x1="2"
            x2="98"
            y1={HOR}
            y2={HOR}
            stroke="var(--muted-foreground)"
            stop-opacity=".5"
            vector-effect="non-scaling-stroke"
          />
          <Show when={matched()}>
            {(m) => (
              <line
                x1={cx()}
                y1={cy()}
                x2={Xaz(m().solar.azimuth)}
                y2={Yp(m().solar.altitude)}
                stroke="var(--success)"
                stop-opacity=".55"
                stroke-dasharray="3 4"
                vector-effect="non-scaling-stroke"
              />
            )}
          </Show>
        </svg>

        <For each={props.wallpapers}>
          {(wp, i) => {
            const sel = createMemo(() => wp.index === props.selIndex),
              mat = createMemo(() => i() === props.matchedEntry),
              hov = createMemo(() => i() === hover());
            return (
              <button
                type="button"
                title={`${t("stage.altitude")} ${wp.solar.altitude}° · ${t("stage.azimuth")} ${wp.solar.azimuth}°`}
                onClick={() => props.onSelect(wp.index)}
                onMouseEnter={() => setHover(i())}
                onMouseLeave={() => setHover(null)}
                style={{
                  left: `${Xaz(wp.solar.azimuth)}%`,
                  top: `${Yp(wp.solar.altitude)}%`,
                }}
                class={clsx(
                  "absolute -translate-x-1/2 -translate-y-1/2 rounded-full ring-1 ring-card transition-all",
                  band(wp.solar.altitude),
                  hov() || sel() ? "size-2.5" : "size-1.75",
                  mat() &&
                    "shadow-[0_0_0_2px_var(--card),0_0_0_3.5px_var(--success)]",
                  sel() &&
                    !mat() &&
                    "shadow-[0_0_0_2px_var(--card),0_0_0_3.5px_var(--success)]",
                )}
              />
            );
          }}
        </For>

        <div
          style={{ left: `${cx()}%`, top: `${cy()}%` }}
          class={clsx(
            "absolute size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full",
            props.current.altitude < 0
              ? "bg-muted-foreground shadow-[0_0_10px_var(--muted-foreground)]"
              : "bg-warning shadow-[0_0_12px_var(--warning)]",
          )}
        />

        {/* 地平线标注：置于中段空位（此处日轨高悬，无轨道经过），不再压到东/西端交点 */}
        <span
          class="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 rounded bg-card/70 px-1 font-mono text-[8px] text-muted-foreground"
          style={{ top: `${HOR}%` }}
        >
          {t("stage.horizon")}
        </span>
        <span
          class="absolute left-2 -translate-y-1/2 font-mono text-[8px] text-muted-foreground"
          style={{ top: `${Y_BOT}%` }}
        >
          {t("stage.minus20")}
        </span>
        {[
          { az: 90, label: t("stage.east") },
          { az: 180, label: t("stage.south") },
          { az: 270, label: t("stage.west") },
        ].map((m) => (
          <span
            class="absolute bottom-1 -translate-x-1/2 font-mono text-[9px] text-muted-foreground"
            style={{ left: `${Xaz(m.az)}%` }}
          >
            {m.label}
          </span>
        ))}
        <span class="absolute right-2 top-1.5 font-mono text-[9px] text-muted-foreground">
          {props.current.altitude < 0 ? "☾" : "☀"} {props.current.altitude}° ·{" "}
          {props.current.azimuth}°
        </span>
      </div>
    </div>
  );
}

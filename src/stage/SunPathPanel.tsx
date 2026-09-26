// 职责：主题太阳轨迹预览——轨迹为「观测者位置 + 当日日期」下的真实日轨（Rust 计算），
//       主题的太阳角条目仅作为标记点叠加其上；Y 轴自适应全部节点。
import type { SolarPosition, Wallpaper } from "@/domain/types";
import { getSolarPath } from "@/ipc";
import { t } from "@/i18n";
import { settingsStore } from "@/store/settings.store";
import { clsx, formatAngle } from "@/utils";
import { createMemo, createResource, createSignal, For, Show } from "solid-js";
import { themeStore } from "~/store/theme.store";
import { type Point, smoothPath, xForAzimuth } from "./sunPath";

// 垂直绘图区（百分比）：上留信息条带，下留方位标签条带。
const Y_TOP = 10,
  Y_BOT = 84;
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

  // 真实日轨：随观测者位置变化重算（当日日期）。
  const [path] = createResource(
    () => settingsStore.config?.position_source,
    (ps) => getSolarPath(ps, Math.floor(Date.now() / 1000)),
  );

  // Y 轴自适应：以「真实日轨」为基准（与主题无关，故各主题看到的是同一条轨迹），
  // 并覆盖当前位置与地平线(0)°，两侧留 8% 边距。轨迹未加载时用主题角度兜底。
  const domain = createMemo(() => {
    const pathAlts = (path() ?? []).map((p) => p.altitude);
    const alts = [
      props.current.altitude,
      0,
      ...pathAlts,
      ...(pathAlts.length === 0
        ? props.wallpapers.map((w) => w.solar.altitude)
        : []),
    ];
    const lo = Math.min(...alts);
    const hi = Math.max(...alts);
    const pad = Math.max((hi - lo) * 0.08, 4);
    return { lo: lo - pad, hi: hi + pad };
  });
  const Yp = (alt: number) => {
    const { lo, hi } = domain();
    return Y_TOP + ((hi - alt) / (hi - lo)) * (Y_BOT - Y_TOP);
  };
  const hor = () => Yp(0);

  /** 纵坐标夹在绘图区内：主题角度超出当日日轨范围时，标记仍可见、可选中。 */
  const clampY = (alt: number) => Math.min(Math.max(Yp(alt), Y_TOP), Y_BOT);

  const cx = createMemo(() => xForAzimuth(props.current.azimuth));
  const cy = createMemo(() => Yp(props.current.altitude));

  const matched = createMemo(() =>
    props.matchedEntry != null
      ? props.wallpapers[props.matchedEntry]
      : undefined,
  );

  const track = createMemo(() =>
    smoothPath(
      (path() ?? []).map(
        (p): Point => [xForAzimuth(p.azimuth), Yp(p.altitude)],
      ),
    ),
  );

  return (
    <div class="mx-auto w-full max-w-155 shrink-0">
      <div class="relative h-28 shrink-0 overflow-hidden rounded-xl border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_6px_18px_rgba(0,0,0,.06)]">
        <div
          class={clsx(
            "pointer-events-none absolute inset-x-0 top-0",
            themeStore.resolved === "light" &&
              "bg-[radial-gradient(120%_80%_at_50%_-10%,color-mix(in_oklch,var(--color-sky-500),transparent_5%),transparent_60%)]",
          )}
          style={{ height: `${hor()}%` }}
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

          <Show when={track()}>
            {(d) => (
              <path
                d={d()}
                fill="none"
                stroke="url(#spArc)"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                opacity=".8"
                vector-effect="non-scaling-stroke"
              />
            )}
          </Show>
          <rect
            x="0"
            y={hor()}
            width="100"
            height={100 - hor()}
            fill="var(--card)"
            opacity=".45"
          />
          <rect
            x="0"
            y={hor()}
            width="100"
            height={100 - hor()}
            fill="url(#spGround)"
          />
          <line
            x1="2"
            x2="98"
            y1={hor()}
            y2={hor()}
            stroke="var(--muted-foreground)"
            stroke-opacity=".5"
            vector-effect="non-scaling-stroke"
          />
          <Show when={matched()}>
            {(m) => (
              <line
                x1={cx()}
                y1={cy()}
                x2={xForAzimuth(m().solar.azimuth)}
                y2={clampY(m().solar.altitude)}
                stroke="var(--success)"
                stroke-opacity=".55"
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
                title={`${t("stage.altitude")} ${formatAngle(wp.solar.altitude)}° · ${t("stage.azimuth")} ${formatAngle(wp.solar.azimuth)}°`}
                onClick={() => props.onSelect(wp.index)}
                onMouseEnter={() => setHover(i())}
                onMouseLeave={() => setHover(null)}
                style={{
                  left: `${xForAzimuth(wp.solar.azimuth)}%`,
                  top: `${clampY(wp.solar.altitude)}%`,
                }}
                class={clsx(
                  "absolute -translate-x-1/2 -translate-y-1/2 rounded-full ring-1 ring-card transition-all",
                  band(wp.solar.altitude),
                  hov() || sel() ? "size-2.5" : "size-1.75",
                  (mat() || sel()) &&
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

        {/* 地平线标注：置于中段空位（此处日轨高悬，无轨道经过） */}
        <span
          class="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 rounded bg-card/70 px-1 font-mono text-[8px] text-muted-foreground"
          style={{ top: `${hor()}%` }}
        >
          {t("stage.horizon")}
        </span>
        <span
          class="absolute left-2 -translate-y-1/2 font-mono text-[8px] text-muted-foreground"
          style={{ top: `${Y_BOT}%` }}
        >
          {Math.round(domain().lo)}°
        </span>
        {[
          { az: 90, label: t("stage.east") },
          { az: 180, label: t("stage.south") },
          { az: 270, label: t("stage.west") },
        ].map((m) => (
          <span
            class="absolute bottom-1 -translate-x-1/2 font-mono text-[9px] text-muted-foreground"
            style={{ left: `${xForAzimuth(m.az)}%` }}
          >
            {m.label}
          </span>
        ))}
        <span class="absolute right-2 top-1.5 font-mono text-[9px] text-muted-foreground">
          {props.current.altitude < 0 ? "☾" : "☀"}{" "}
          {formatAngle(props.current.altitude)}° ·{" "}
          {formatAngle(props.current.azimuth)}°
        </span>
      </div>
    </div>
  );
}

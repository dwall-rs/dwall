// Responsibility: the theme's frames as a horizontal contact sheet. The frame the sun
// matched is always marked, and the one being previewed is brought into view.
import type { Wallpaper } from "@/models/types";
import { t } from "@/i18n";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { clsx, formatAngle } from "@/utils";
import { createEffect, For, Show } from "solid-js";

interface Props {
  themeId: string;
  wallpapers: Wallpaper[];
  matchedIndex: number | null;
  selIndex: number | null;
  onSelect: (index: number | null) => void;
}

export function WallpaperRail(props: Props) {
  let scroller: HTMLDivElement | undefined;

  createEffect(() => {
    const index = props.selIndex ?? props.matchedIndex;
    if (index == null || !scroller) return;
    const card = scroller.querySelector<HTMLElement>(`[data-frame="${index}"]`);
    card?.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "center",
    });
  });

  return (
    <section aria-label={t("workspace.frames")} class="min-w-0">
      <div class="mb-2.5 flex items-baseline justify-between gap-3">
        <h2 class="text-[12px] font-semibold text-foreground">
          {t("workspace.frames")}
        </h2>
        <span class="font-mono text-[10.5px] text-muted-foreground">
          {t("workspace.frameCount", { n: props.wallpapers.length })}
        </span>
      </div>

      <div
        ref={scroller}
        class="flex gap-2 overflow-x-auto pb-2 [scrollbar-width:thin]"
      >
        <For each={props.wallpapers}>
          {(wp) => {
            const selected = () => wp.index === props.selIndex;
            const matched = () => wp.index === props.matchedIndex;
            return (
              <button
                type="button"
                data-frame={wp.index}
                aria-pressed={selected()}
                onClick={() => props.onSelect(selected() ? null : wp.index)}
                class={clsx(
                  "group relative h-[72px] w-[124px] shrink-0 cursor-pointer overflow-hidden rounded-[10px] border text-left transition-all outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                  selected()
                    ? "border-primary shadow-[0_0_0_2px_color-mix(in_oklch,var(--primary),transparent_72%)]"
                    : matched()
                      ? "border-warning/70"
                      : "border-border hover:border-border-2",
                )}
              >
                <ThemeThumbnail
                  themeId={props.themeId}
                  index={wp.index}
                  class="transition-transform duration-300 group-hover:scale-[1.03]"
                />
                <span class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/82 to-transparent px-1.5 pb-1 pt-3 font-mono text-[8.5px] text-white/85">
                  {formatAngle(wp.solar.altitude)}° ·{" "}
                  {formatAngle(wp.solar.azimuth)}°
                </span>
                <Show when={matched()}>
                  <span class="absolute left-1 top-1 rounded-[3px] bg-warning px-1 py-px font-mono text-[7.5px] font-bold tracking-wide text-warning-foreground">
                    {t("stage.match")}
                  </span>
                </Show>
                <Show when={selected()}>
                  <span class="absolute right-1 top-1 size-1.5 rounded-full bg-primary ring-2 ring-black/25" />
                </Show>
              </button>
            );
          }}
        </For>
      </div>
    </section>
  );
}

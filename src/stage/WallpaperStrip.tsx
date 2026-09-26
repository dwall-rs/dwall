/* ===== src/stage/WallpaperStrip.tsx ===== */
// 职责：壁纸条——数量可变、定宽；以缩略图展示，标注各自太阳位置；匹配项醒目标记，点击选中/取消。
import type { Wallpaper } from "@/domain/types";
import { t } from "@/i18n";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { clsx, formatAngle } from "@/utils";

interface Props {
  themeId: string;
  wallpapers: Wallpaper[];
  matchedIndex: number | null;
  selIndex: number | null;
  onSelect: (index: number | null) => void;
}

export function WallpaperStrip(props: Props) {
  return (
    <div class="shrink-0">
      <div class="mb-[9px] flex items-center justify-between">
        <span class="font-display text-[11px] font-bold uppercase tracking-[1.2px] text-muted-foreground">
          {t("stage.stripTitle")}
        </span>
        <span class="hidden font-mono text-[10.5px] text-muted-foreground min-[1150px]:block">
          {t("stage.stripHint")}
        </span>
      </div>
      <div class="flex gap-[7px] overflow-x-auto pb-1.5">
        {props.wallpapers.map((wp, i) => {
          const sel = wp.index === props.selIndex;
          const matched = wp.index === props.matchedIndex;
          return (
            <div
              onClick={() => props.onSelect(sel ? null : wp.index)}
              style={{ width: "104px", "animation-delay": `${i * 40}ms` }}
              class={clsx(
                "group relative h-[64px] shrink-0 cursor-pointer overflow-hidden rounded-lg border transition-all duration-200 animate-rise hover:-translate-y-0.5",
                sel
                  ? "border-primary ring-2 ring-primary/30"
                  : matched
                    ? "border-success ring-2 ring-success/30"
                    : "border-border hover:border-border-2",
              )}
            >
              <ThemeThumbnail themeId={props.themeId} index={wp.index} />
              {matched && (
                <span class="absolute inset-x-0 top-0 flex items-center justify-center gap-1 bg-success py-0.5 font-mono text-[9px] font-bold tracking-wide text-success-foreground">
                  {t("stage.match")}
                </span>
              )}
              <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent py-0.5 text-center font-mono text-[8.5px] text-white">
                {formatAngle(wp.solar.altitude)}° ·{" "}
                {formatAngle(wp.solar.azimuth)}°
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

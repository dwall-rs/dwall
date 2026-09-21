/* ===== src/stage/WallpaperStrip.tsx ===== */
// 职责：壁纸条——数量可变、定宽；以缩略图展示，标注各自太阳位置；选中/匹配高亮。
import type { Wallpaper } from "@/domain/types";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { clsx } from "@/utils";

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
          套内壁纸 · 按太阳位置匹配
        </span>
        <span class="hidden font-mono text-[10.5px] text-muted-foreground min-[1150px]:block">
          点击预览该太阳位置下的壁纸
        </span>
      </div>
      <div class="flex gap-[7px] overflow-x-auto pb-1.5">
        {/* 「回到当前匹配」快捷项 */}
        <button
          type="button"
          onClick={() => props.onSelect(null)}
          class={clsx(
            "flex h-[64px] shrink-0 items-center gap-2 rounded-lg border px-3 font-mono text-[11px] transition-all animate-rise",
            props.selIndex === null
              ? "border-primary bg-primary/10 text-primary"
              : "border-border text-muted-foreground hover:border-border-2",
          )}
        >
          <span class="size-1.5 rounded-full bg-success animate-bdot" />
          当前匹配
        </button>
        {props.wallpapers.map((wp, i) => {
          const sel = wp.index === props.selIndex;
          const matched = wp.index === props.matchedIndex;
          return (
            <div
              onClick={() => props.onSelect(wp.index)}
              style={{ width: "104px", "animation-delay": `${i * 40}ms` }}
              class={clsx(
                "group relative h-[64px] shrink-0 cursor-pointer overflow-hidden rounded-lg border transition-all duration-200 animate-rise hover:-translate-y-0.5",
                sel
                  ? "border-primary shadow-[0_0_0_2px_hsl(var(--primary)/.34)]"
                  : "border-border hover:border-border-2",
              )}
            >
              <ThemeThumbnail themeId={props.themeId} index={wp.index} />
              {matched && (
                <span class="absolute left-1 top-1 rounded bg-success px-1 font-mono text-[9px] leading-4 text-success-foreground">
                  匹配
                </span>
              )}
              <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent py-0.5 text-center font-mono text-[8.5px] text-white">
                {wp.solar.altitude}° · {wp.solar.azimuth}°
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

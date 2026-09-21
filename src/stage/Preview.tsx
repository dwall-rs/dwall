/* ===== src/stage/Preview.tsx ===== */
// 职责：主预览——contain 保留壁纸原生比例；标注太阳位置与是否匹配。
import type { Wallpaper } from "@/domain/types";
import { WallpaperImage } from "@/scene/WallpaperImage";

interface Props {
  wallpaper: Wallpaper;
  isMatched: boolean;
}

export function Preview(props: Props) {
  return (
    <div class="relative min-h-0 flex-1 overflow-hidden rounded-2xl border border-foreground/10 bg-black shadow-[0_24px_60px_rgba(0,0,0,.5)]">
      <div class="absolute inset-3 overflow-hidden rounded-[12px]">
        <WallpaperImage path={props.wallpaper.path} fit="contain" />
      </div>
      <div class="absolute left-4 top-4 flex items-center gap-2 rounded-[11px] border border-white/15 bg-black/50 px-3.25 py-2 text-white backdrop-blur-md">
        <span class="font-mono text-[11px] text-white/70">
          高度 {props.wallpaper.solar.altitude}° · 方位{" "}
          {props.wallpaper.solar.azimuth}°
        </span>
        <span class="font-mono text-[11px] text-white/50">
          #{props.wallpaper.index + 1}
        </span>
        {props.isMatched && (
          <span class="flex items-center gap-1 font-mono text-[10px] text-success">
            <span class="size-1.5 rounded-full bg-success animate-bdot" />
            当前匹配
          </span>
        )}
      </div>
    </div>
  );
}

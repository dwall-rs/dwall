/* ===== src/components/stage/Preview.tsx ===== */
// 职责：主预览——contain 保留壁纸原生比例（留白即比例差异的直观体现），标注太阳位置与是否匹配。
import type { SceneType, Wallpaper } from "@/domain/types";
import { Scene } from "@/scene/Scene";
import { createMemo } from "solid-js";

interface Props {
  type: SceneType;
  wallpaper: Wallpaper;
  isMatched: boolean;
}

export function Preview(props: Props) {
  const ratio = createMemo(() =>
    (props.wallpaper.w / props.wallpaper.h).toFixed(2),
  );
  return (
    <div class="relative min-h-0 flex-1 overflow-hidden rounded-2xl border border-foreground/10 bg-black shadow-[0_24px_60px_rgba(0,0,0,.5)]">
      <div class="absolute inset-3 overflow-hidden rounded-[12px]">
        <Scene
          type={props.type}
          solar={props.wallpaper.solar}
          w={props.wallpaper.w}
          h={props.wallpaper.h}
          fit="contain"
        />
      </div>
      <div class="absolute left-4 top-4 flex items-center gap-2 rounded-[11px] border border-white/15 bg-black/50 px-3.25 py-2 text-white backdrop-blur-md">
        <span class="font-mono text-[11px] text-white/70">
          高度 {props.wallpaper.solar.altitude}° · 方位{" "}
          {props.wallpaper.solar.azimuth}°
        </span>
        <span class="font-mono text-[11px] text-white/50">
          {props.wallpaper.w}×{props.wallpaper.h} · {ratio()}:1
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

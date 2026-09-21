/* ===== src/stage/PoolCollage.tsx ===== */
// 职责：候选池拼贴墙——仅预览池内容，明确标注「非抽中结果」。
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

const LAYOUT = ["big", "", "tall", "", "", "", "tall", "", "big", ""];

export function PoolCollage() {
  const pool = themeList()
    .filter((t) => randomStore.selected.includes(t.id))
    .slice(0, 10);
  return (
    <div class="relative grid min-h-0 flex-1 grid-cols-4 auto-rows-fr gap-[3px] overflow-hidden rounded-[18px] border border-border-2 bg-black shadow-[0_24px_60px_rgba(0,0,0,.45)]">
      {pool.map((t, i) => (
        <div
          class={`relative overflow-hidden ${LAYOUT[i] === "big" ? "col-span-2 row-span-2" : LAYOUT[i] === "tall" ? "row-span-2" : ""}`}
        >
          <ThemeThumbnail
            themeId={t.id}
            class="saturate-[.92] transition-transform duration-500 hover:scale-105"
          />
        </div>
      ))}
      <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/72 via-transparent to-transparent" />
      <div class="pointer-events-none absolute bottom-4 left-[18px] right-[18px] text-white">
        <div class="font-display text-[15px] font-bold">候选池拼贴</div>
        <div class="mt-[3px] font-mono text-[11px] text-white/70">
          {pool.length} 套参与每日洗牌 · 仅预览池内容，非抽中结果
        </div>
      </div>
    </div>
  );
}

/* ===== src/components/stage/RandomStage.tsx ===== */
// 职责：随机模式舞台编排——导语 + 摘要 + 拼贴/空态。无任何可调控件（编辑器不抽样）。
import { randomStore } from "@/store/random.store";
import { RandomStats } from "./RandomStats";
import { PoolCollage } from "./PoolCollage";
import { PoolEmpty } from "./PoolEmpty";

export function RandomStage() {
  return (
    <>
      <p class="max-w-[580px] font-display text-[15px] font-semibold leading-relaxed text-foreground">
        每天从候选池<em class="not-italic text-warning">洗牌</em>抽取一套，
        <em class="not-italic text-warning">同时应用到所有显示器</em>
        。编辑器只负责选定池子，抽中哪套由引擎在运行时决定。
      </p>
      <RandomStats />
      {randomStore.selected.length === 0 ? <PoolEmpty /> : <PoolCollage />}
    </>
  );
}

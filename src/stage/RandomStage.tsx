/* ===== src/components/stage/RandomStage.tsx ===== */
// 职责：随机模式舞台编排——导语 + 摘要 + 拼贴/空态。无任何可调控件（编辑器不抽样）。
import { randomStore } from "@/store/random.store";
import { t } from "@/i18n";
import { RandomStats } from "./RandomStats";
import { PoolCollage } from "./PoolCollage";
import { PoolEmpty } from "./PoolEmpty";

export function RandomStage() {
  return (
    <>
      <p class="max-w-[580px] font-display text-[15px] font-semibold leading-relaxed text-foreground">
        {t("stage.randomIntro")}
      </p>
      <RandomStats />
      {randomStore.selected.length === 0 ? <PoolEmpty /> : <PoolCollage />}
    </>
  );
}

/* ===== src/components/stage/RandomStage.tsx ===== */
// 职责：随机模式舞台编排——导语 + 池状态 + 摘要 + 拼贴/空态。无任何可调控件（编辑器不抽样）。
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { createMemo } from "solid-js";
import { WriteNote } from "./WriteNote";
import { RandomStats } from "./RandomStats";
import { PoolCollage } from "./PoolCollage";
import { PoolEmpty } from "./PoolEmpty";

export function RandomStage() {
  // 以「目录中真实存在」的主题数为准，避免池里只剩已卸载 id 时仍渲染空拼贴。
  const poolSize = createMemo(
    () =>
      themeList().filter((theme) => randomStore.selected.includes(theme.id))
        .length,
  );

  return (
    <>
      <p class="max-w-[580px] font-display text-[15px] font-semibold leading-relaxed text-foreground">
        {t("stage.randomIntro")}
      </p>
      <WriteNote />
      <RandomStats />
      {poolSize() === 0 ? <PoolEmpty /> : <PoolCollage />}
    </>
  );
}

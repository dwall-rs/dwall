// Responsibility: Random-mode stage orchestration — today's sun track, pool status, summary + select all /
//       clear all, then the collage or the empty state. No sampling.
import { randomStore, selectAll, clearAll } from "@/store/random.store";
import { availableThemeIds } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { createMemo, Show } from "solid-js";
import { WriteNote } from "./WriteNote";
import { RandomStats } from "./RandomStats";
import { RandomSunPath } from "./RandomSunPath";
import { PoolCollage } from "./PoolCollage";
import { PoolEmpty } from "./PoolEmpty";
import { RandomCommitRow } from "./RandomCommitRow";

export function RandomStage() {
  // Only themes on disk get a turn, so "select all" and the counts work from those.
  const poolIds = createMemo(() => availableThemeIds());
  const poolSize = createMemo(() => randomStore.selected.length);

  return (
    <>
      {/* The track leads: it is the only part of this stage that moves during the day. An
          empty pool has nothing drawn, so it stays out of the way then. */}
      <Show when={poolSize() > 0}>
        <RandomSunPath />
      </Show>
      <WriteNote />
      <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
        <RandomStats />
        {/* The explanation sits in the space the figures leave, at caption size: it is a
            note about the mode, not a heading over the stage. */}
        <p class="min-w-0 max-w-120 flex-1 text-[11.5px] leading-relaxed text-muted-foreground">
          {t("stage.randomIntro")}
        </p>
        <div class="ml-auto flex items-center gap-4 text-[12px]">
          <button
            type="button"
            disabled={poolSize() === poolIds().length}
            onClick={() => selectAll(poolIds())}
            class="cursor-pointer text-primary transition-opacity hover:underline disabled:cursor-default disabled:opacity-40 disabled:hover:no-underline"
          >
            {t("scope.selectAll")}
          </button>
          <button
            type="button"
            disabled={poolSize() === 0}
            onClick={clearAll}
            class="cursor-pointer text-muted-foreground transition-colors hover:text-foreground hover:underline disabled:cursor-default disabled:opacity-40 disabled:hover:text-muted-foreground disabled:hover:no-underline"
          >
            {t("scope.clearAll")}
          </button>
        </div>
      </div>
      {poolSize() === 0 ? <PoolEmpty /> : <PoolCollage />}
      <RandomCommitRow />
    </>
  );
}

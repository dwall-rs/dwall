// Responsibility: Random-mode stage orchestration — intro + pool status + summary + select all / clear all + collage or empty state. No sampling.
import { randomStore, selectAll, clearAll } from "@/store/random.store";
import { availableThemeIds } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { createMemo } from "solid-js";
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
      <p class="max-w-145 font-display text-[15px] font-semibold leading-relaxed text-foreground">
        {t("stage.randomIntro")}
      </p>
      <WriteNote />
      <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
        <RandomStats />
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
      {poolSize() === 0 ? (
        <PoolEmpty />
      ) : (
        <>
          {/* The pool area reports rather than asks, so the track sits with it: it belongs
              to the theme drawn for today, not to the whole pool. */}
          <RandomSunPath />
          <PoolCollage />
        </>
      )}
      <RandomCommitRow />
    </>
  );
}

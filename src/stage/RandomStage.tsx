// Responsibility: Random-mode stage orchestration — intro + pool status + summary + select all / clear all + collage or empty state. No sampling.
import { randomStore, selectAll, clearAll } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { createMemo } from "solid-js";
import { WriteNote } from "./WriteNote";
import { RandomStats } from "./RandomStats";
import { PoolCollage } from "./PoolCollage";
import { PoolEmpty } from "./PoolEmpty";
import { RandomCommitRow } from "./RandomCommitRow";

export function RandomStage() {
  const allIds = createMemo(() => themeList().map((theme) => theme.id));
  // Counts only themes that really exist in the catalog, so we never render an empty collage when the pool is left holding uninstalled ids.
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
      <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
        <RandomStats />
        <div class="ml-auto flex items-center gap-4 text-[12px]">
          <button
            type="button"
            disabled={poolSize() === themeList().length}
            onClick={() => selectAll(allIds())}
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

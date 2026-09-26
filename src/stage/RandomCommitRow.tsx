/* ===== src/stage/RandomCommitRow.tsx ===== */
// 职责：随机模式的提交行——「保存配置/放弃」就是随机模式的「应用」，写配置 + 重启引擎。
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { randomStore, isDirty, save, discard } from "@/store/random.store";
import { clsx } from "@/utils";
import { LoaderCircle, Save } from "lucide-solid";
import { createMemo } from "solid-js";

export function RandomCommitRow() {
  const dirty = createMemo(() => isDirty(randomStore));
  const canSave = createMemo(
    () => !randomStore.saving && randomStore.selected.length > 0,
  );

  return (
    <div class="flex shrink-0 items-center justify-center gap-3 pb-4 pt-1">
      <Button variant="ghost" size="sm" disabled={!dirty()} onClick={discard}>
        {t("commit.discard")}
      </Button>
      <Button
        size="sm"
        disabled={!canSave()}
        onClick={save}
        class={clsx(
          dirty() && canSave() && "animate-breathe",
          "px-7 py-2.5 text-[13.5px]",
        )}
      >
        {randomStore.saving ? (
          <>
            <LoaderCircle class="size-3.5 animate-spin" />
            {t("commit.saving")}
          </>
        ) : (
          <>
            <Save class="size-3.5" />
            {t("commit.save")}
          </>
        )}
      </Button>
    </div>
  );
}

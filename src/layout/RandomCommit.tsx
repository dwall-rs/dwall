// 职责：随机模式提交视图——脏态旗 + 保存/放弃 + 三帧（写入中→已保存）。
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { randomStore, isDirty, save, discard } from "@/store/random.store";
import { clsx } from "@/utils";
import { Check, LoaderCircle, Save } from "lucide-solid";
import { createMemo } from "solid-js";

function fmt(timestamp: number) {
  const d = new Date(timestamp);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

export function RandomCommit() {
  const dirty = createMemo(() => isDirty(randomStore));
  return (
    <div class="flex items-center gap-2.5">
      <span
        class={clsx(
          "flex items-center gap-1.5 text-[12.5px] text-warning transition-all duration-300",
          dirty()
            ? "opacity-100"
            : "pointer-events-none translate-x-2 opacity-0",
        )}
      >
        <span class="size-1.75 rounded-full bg-warning animate-bdot" />
        {t("commit.unsaved")}
      </span>
      {!dirty() && (
        <span class="flex items-center gap-1.5 font-mono text-[12px] text-muted-foreground">
          <Check class="size-3.25 text-success" />
          {randomStore.savedAt
            ? t("commit.savedAt", { time: fmt(randomStore.savedAt) })
            : t("commit.saved")}
        </span>
      )}
      <Button variant="ghost" size="sm" disabled={!dirty()} onClick={discard}>
        {t("commit.discard")}
      </Button>
      <Button
        size="sm"
        disabled={
          !dirty() || randomStore.saving || randomStore.selected.length === 0
        }
        onClick={save}
        class={clsx(
          dirty() &&
            randomStore.selected.length > 0 &&
            !randomStore.saving &&
            "animate-breathe",
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

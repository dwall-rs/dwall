// 职责：随机模式提交状态——顶栏只显示「未保存/已保存」；保存动作在舞台的提交行。
import { t } from "@/i18n";
import { randomStore, isDirty } from "@/store/random.store";
import { clsx } from "@/utils";
import { Check } from "lucide-solid";
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
    </div>
  );
}

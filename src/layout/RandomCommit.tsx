// 职责：随机模式提交视图——脏态旗 + 保存/放弃 + 三帧（写入中→已保存）。绑定 Ctrl/⌘+S 不在此（在 GlobalHotkeys）。
import { Button } from "@/components/ui/button";
import { randomStore, isDirty, save, discard } from "@/store/random.store";
import { clsx } from "@/utils";
import { Check, LoaderCircle, Save } from "lucide-solid";
import { createMemo } from "solid-js";

function fmt(t: number) {
  const d = new Date(t);
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
        未保存的更改
      </span>
      {!dirty() && (
        <span class="flex items-center gap-1.5 font-mono text-[12px] text-muted-foreground">
          <Check class="size-3.25 text-success" />
          {randomStore.savedAt
            ? `已保存 ${fmt(randomStore.savedAt)}`
            : "已保存"}
        </span>
      )}
      <Button variant="ghost" size="sm" disabled={!dirty()} onClick={discard}>
        放弃
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
            写入中…
          </>
        ) : (
          <>
            <Save class="size-3.5" />
            保存配置
          </>
        )}
      </Button>
    </div>
  );
}

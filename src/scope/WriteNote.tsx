/* ===== src/components/scope/WriteNote.tsx ===== */
// 职责：随机模式归属/状态条——三态：空集(rose) / 脏态(warning) / 正常(muted)。
import { randomStore, isDirty } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function WriteNote() {
  const dirty = createMemo(() => isDirty(randomStore));
  const n = createMemo(() => randomStore.selected.length);
  const empty = createMemo(() => n() === 0);
  const text = createMemo(() =>
    empty()
      ? "候选池为空，引擎将无壁纸可抽，请至少保留一套。"
      : dirty()
        ? `候选池 ${randomStore.selected.length} 套 · 已排除 ${themeList().length - randomStore.selected.length} · 未保存`
        : `候选池 ${randomStore.selected.length} 套 · 取消勾选即从每日洗牌中排除`,
  );
  return (
    <div
      class={clsx(
        "mx-1.5 mb-3 mt-0.5 flex items-start gap-2 rounded-[7px] border p-[10px_12px] text-[11.5px] leading-relaxed transition-all duration-300",
        empty()
          ? "border-destructive/30 bg-destructive-dim text-destructive"
          : dirty()
            ? "border-warning/30 bg-warning-dim text-warning"
            : "border-border-2 bg-secondary text-muted-foreground",
      )}
    >
      <span
        class={clsx(
          "mt-1.25 size-1.5 shrink-0 rounded-full",
          empty()
            ? "bg-destructive"
            : dirty()
              ? "bg-warning animate-bdot"
              : "bg-muted-foreground",
        )}
      />
      <span>{text()}</span>
    </div>
  );
}

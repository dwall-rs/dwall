/* ===== src/stage/WriteNote.tsx ===== */
// 职责：随机模式候选池状态条——三态：空集(destructive) / 脏态(warning) / 正常(muted)。
import { randomStore, isDirty } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function WriteNote() {
  const dirty = createMemo(() => isDirty(randomStore));
  const n = createMemo(() => randomStore.selected.length);
  const empty = createMemo(() => n() === 0);
  const text = createMemo(() =>
    empty()
      ? t("scope.randomEmpty")
      : dirty()
        ? t("scope.randomDirty", {
            n: randomStore.selected.length,
            excluded: themeList().length - randomStore.selected.length,
          })
        : t("scope.randomNormal", { n: randomStore.selected.length }),
  );
  return (
    <div
      class={clsx(
        "flex items-start gap-2 rounded-[7px] border p-[10px_12px] text-[11.5px] leading-relaxed transition-all duration-300",
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

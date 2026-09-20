/* ===== src/components/stage/PoolEmpty.tsx ===== */
// 职责：候选池空态——引导至少保留一套。

import { ImageOff } from "lucide-solid";

export function PoolEmpty() {
  return (
    <div class="relative flex min-h-0 flex-1 flex-col items-center justify-center gap-2.5 rounded-[18px] border border-dashed border-border-2 text-muted-foreground">
      <ImageOff class="size-10 opacity-40" />
      <div class="font-display text-[15px] font-bold text-foreground/80">
        候选池为空
      </div>
      <div class="max-w-[300px] text-center text-[12px] leading-relaxed">
        你已排除全部主题。请在左侧至少保留一套，引擎才有壁纸可抽。
      </div>
    </div>
  );
}

/* ===== src/components/scope/ThemeCheckItem.tsx ===== */
// 职责：随机模式单个候选主题勾选行。
import { randomStore, toggle } from "@/store/random.store";
import { themeById } from "@/domain/themes";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

interface Props {
  id: string;
}

export function ThemeCheckItem({ id }: Props) {
  const on = createMemo(() => randomStore.selected.includes(id));
  const t = themeById(id);
  return (
    <div
      onClick={() => toggle(id)}
      class="flex cursor-pointer items-center gap-2.5 rounded-[7px] p-2 transition-all duration-200 hover:translate-x-0.5 hover:bg-secondary"
    >
      <div class="h-7 w-10 shrink-0 overflow-hidden rounded-md shadow-[0_2px_6px_rgba(0,0,0,.4)]">
        <ThemeThumbnail themeId={id} />
      </div>
      <div
        class={clsx(
          "flex-1 truncate text-[12.5px]",
          on() ? "font-medium text-foreground" : "text-muted-foreground",
        )}
      >
        {t.name}
      </div>
      <span
        class={clsx(
          "relative size-4 shrink-0 rounded-[5px] border-[1.5px] transition-all",
          on() ? "border-warning bg-warning" : "border-muted-foreground",
        )}
      >
        {on() && (
          <span class="absolute left-[4.5px] top-[1.5px] h-2 w-1 rotate-45 border-b-2 border-r-2 border-warning-foreground" />
        )}
      </span>
    </div>
  );
}

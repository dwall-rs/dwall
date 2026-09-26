// 职责：随机模式作用域面板——归属条 + 搜索 + 候选勾选列表（无 Radio，默认全选）。
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { WriteNote } from "./WriteNote";
import { ThemeCheckItem } from "./ThemeCheckItem";
import { Search } from "lucide-solid";
import { createMemo, createSignal } from "solid-js";

export function RandomScope() {
  const [q, setQ] = createSignal("");
  const list = createMemo(() =>
    themeList().filter((t) => t.name.toLowerCase().includes(q().toLowerCase())),
  );
  return (
    <>
      <WriteNote />
      <div class="relative mx-1.5 mb-2.5">
        <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <input
          value={q()}
          onInput={(e) => setQ(e.currentTarget.value)}
          placeholder={t("scope.searchThemes")}
          class="w-full rounded-lg border border-border-2 bg-secondary py-2.25 pl-8 pr-3 text-[12.5px] text-foreground outline-none transition-colors focus:border-ring"
        />
      </div>
      {list().map((t) => (
        <ThemeCheckItem id={t.id} />
      ))}
    </>
  );
}

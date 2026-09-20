// 职责：外观三档 segmented（亮/暗/系统）；读写 theme.store，pill 用等宽网格定位。
import { themeStore, setMode } from "@/store/theme.store";
import type { ThemeMode } from "@/domain/types";
import { Monitor, Moon, Sun } from "lucide-solid";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { For } from "solid-js";

const ITEMS: { value: ThemeMode; label: string; Icon: typeof Sun }[] = [
  { value: "light", label: "亮", Icon: Sun },
  { value: "dark", label: "暗", Icon: Moon },
  { value: "system", label: "系统", Icon: Monitor },
];

export function ThemeSegmented() {
  const idx = ITEMS.findIndex((i) => i.value === themeStore.mode);

  return (
    <div class="relative rounded-[11px] border border-border-2 bg-secondary p-0.75">
      <span
        class="absolute top-0.75 bottom-0.75 rounded-lg bg-accent shadow-[inset_0_0_0_1px_hsl(var(--border-2))] transition-all duration-300 ease-[cubic-bezier(.22,.61,.36,1)]"
        style={{
          left: `calc(${idx} * (100% - 6px) / 3 + 3px)`,
          width: "calc((100% - 6px) / 3)",
        }}
      />
      <Tabs value={themeStore.mode}>
        <TabsList>
          <For each={ITEMS}>
            {(it) => (
              <TabsTrigger value={it.value} onClick={() => setMode(it.value)}>
                <it.Icon class="size-3.5" />
                {it.label}
              </TabsTrigger>
            )}
          </For>
        </TabsList>
      </Tabs>
    </div>
  );
}

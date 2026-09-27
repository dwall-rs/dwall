// Responsibility: Appearance three-way segmented control (light/dark/system); the active state comes from the UI kit Tabs (data-active).
import { themeStore, setMode } from "@/store/theme.store";
import type { ThemeMode } from "@/domain/types";
import { t } from "@/i18n";
import { Monitor, Moon, Sun } from "lucide-solid";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { For } from "solid-js";

const ITEMS: {
  value: ThemeMode;
  key: "app.appearance.light" | "app.appearance.dark" | "app.appearance.system";
  Icon: typeof Sun;
}[] = [
  { value: "light", key: "app.appearance.light", Icon: Sun },
  { value: "dark", key: "app.appearance.dark", Icon: Moon },
  { value: "system", key: "app.appearance.system", Icon: Monitor },
];

export function ThemeSegmented() {
  return (
    <div class="rounded-[11px] border border-border-2 bg-secondary p-0.75">
      <Tabs value={themeStore.mode}>
        <TabsList>
          <For each={ITEMS}>
            {(it) => (
              <TabsTrigger value={it.value} onClick={() => setMode(it.value)}>
                <it.Icon class="size-3.5" />
                {t(it.key)}
              </TabsTrigger>
            )}
          </For>
        </TabsList>
      </Tabs>
    </div>
  );
}

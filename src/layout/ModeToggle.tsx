// 职责：固定/随机 segmented；读写 ui.store 的 mode。
import { For } from "solid-js";
import { uiStore, setMode } from "@/store/ui.store";
import type { Mode } from "@/domain/types";
import { t } from "@/i18n";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

const ITEMS: { value: Mode; key: "app.mode.fixed" | "app.mode.random" }[] = [
  { value: "fixed", key: "app.mode.fixed" },
  { value: "random", key: "app.mode.random" },
];

export function ModeToggle() {
  return (
    <Tabs value={uiStore.mode}>
      <TabsList>
        <For each={ITEMS}>
          {({ value, key }) => (
            <TabsTrigger value={value} onClick={() => setMode(value)}>
              {t(key)}
            </TabsTrigger>
          )}
        </For>
      </TabsList>
    </Tabs>
  );
}

// Responsibility: Fixed/random segmented control; reads and writes the mode in mode.store.
import { For } from "solid-js";
import { modeStore, setMode } from "@/store/mode.store";
import type { Mode } from "@/domain/types";
import { t } from "@/i18n";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

const ITEMS: { value: Mode; key: "app.mode.fixed" | "app.mode.random" }[] = [
  { value: "fixed", key: "app.mode.fixed" },
  { value: "random", key: "app.mode.random" },
];

export function ModeToggle() {
  return (
    <Tabs value={modeStore.mode}>
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

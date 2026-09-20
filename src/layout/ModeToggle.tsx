// 职责：固定/随机 segmented；读写 ui.store 的 mode。pill 用等宽网格定位，无需测量 DOM。
import { For } from "solid-js";
import { uiStore, setMode } from "@/store/ui.store";
import type { Mode } from "@/domain/types";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

const ITEMS: { value: Mode; label: string }[] = [
  { value: "fixed", label: "固定" },
  { value: "random", label: "随机" },
];

export function ModeToggle() {
  return (
    <Tabs defaultValue={uiStore.mode}>
      <TabsList>
        <For each={ITEMS}>
          {({ value, label }) => (
            <TabsTrigger value={value} onClick={() => setMode(value)}>
              {label}
            </TabsTrigger>
          )}
        </For>
      </TabsList>
    </Tabs>
  );
}

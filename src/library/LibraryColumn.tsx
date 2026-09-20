// 职责：右栏容器；补 h-full 以在抽屉形态下撑满高度。其余不变。
import { uiStore } from "@/store/ui.store";
import { fixedStore } from "@/store/fixed.store";
import { INITIAL_MONITORS } from "@/domain/monitors";
import { THEMES } from "@/domain/themes";
import { ThemeTile } from "./ThemeTile";
import { Search } from "lucide-solid";
import { createMemo, createSignal, For } from "solid-js";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "~/components/ui/input-group";

export function LibraryColumn() {
  const [q, setQ] = createSignal("");
  const list = createMemo(() =>
    THEMES.filter((t) => t.name.toLowerCase().includes(q().toLowerCase())),
  );
  const scopeName = createMemo(() =>
    fixedStore.allUnified
      ? "所有显示器"
      : INITIAL_MONITORS.find((m) => m.id === fixedStore.curMon)!.name,
  );

  return (
    <div class="col-lib flex h-full min-h-0 flex-col border-l border-border">
      <div class="flex items-baseline justify-between px-4.5 pb-2.5 pt-4">
        <span class="font-display text-[11px] font-bold uppercase tracking-[1.4px] text-muted-foreground">
          主题库
        </span>
        <span class="hidden font-mono text-[11px] text-muted-foreground min-[1150px]:block">
          {uiStore.mode === "fixed"
            ? `点击指定给${scopeName()}`
            : "点击加入 / 移出候选池"}
        </span>
      </div>
      <div class="px-3.5 pb-2.5">
        <InputGroup>
          <InputGroupInput
            class="text-xs placeholder:text-xs"
            placeholder="搜索主题…"
            value={q()}
            onChange={(v) => setQ(v)}
          />
          <InputGroupAddon>
            <Search class="size-3.5" />
          </InputGroupAddon>
        </InputGroup>
      </div>
      <div class="scrollbar-thin min-h-0 flex-1 overflow-y-auto px-3.5 pb-4.5">
        <div class="grid grid-cols-2 gap-2.25 pt-0.5">
          <For each={list()}>
            {(t, i) => <ThemeTile id={t.id} index={i()} />}
          </For>
        </div>
      </div>
    </div>
  );
}

// 职责：右栏容器；补 h-full 以在抽屉形态下撑满高度。其余不变。
import { uiStore } from "@/store/ui.store";
import { fixedStore } from "@/store/fixed.store";
import { monitorById } from "@/domain/monitors";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
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
    themeList().filter((theme) =>
      theme.name.toLowerCase().includes(q().toLowerCase()),
    ),
  );
  const scopeName = createMemo(() =>
    fixedStore.allUnified
      ? t("scope.allMonitors")
      : (monitorById(fixedStore.curMon)?.name ?? fixedStore.curMon),
  );

  return (
    <div class="col-lib flex h-full min-h-0 flex-col border-l border-border">
      <div class="flex items-baseline justify-between px-4.5 pb-2.5 pt-4">
        <span class="font-display text-[11px] font-bold uppercase tracking-[1.4px] text-muted-foreground">
          {t("library.title")}
        </span>
        <span class="hidden font-mono text-[11px] text-muted-foreground min-[1150px]:block">
          {uiStore.mode === "fixed"
            ? t("library.hintFixed", { name: scopeName() })
            : t("library.hintRandom")}
        </span>
      </div>
      <div class="px-3.5 pb-2.5">
        <InputGroup>
          <InputGroupInput
            class="text-xs placeholder:text-xs"
            placeholder={t("library.search")}
            value={q()}
            onInput={(v) => setQ(v)}
          />
          <InputGroupAddon>
            <Search class="size-3.5" />
          </InputGroupAddon>
        </InputGroup>
      </div>
      <div class="scrollbar-thin min-h-0 flex-1 overflow-y-auto px-3.5 pb-4.5">
        <div class="grid grid-cols-2 gap-2.25 pt-0.5">
          <For each={list()}>
            {(theme, i) => <ThemeTile id={theme.id} index={i()} />}
          </For>
        </div>
      </div>
    </div>
  );
}

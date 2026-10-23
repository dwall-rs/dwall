// Responsibility: Right-column container; adds h-full so it fills the height in drawer form. Everything else unchanged.
import { modeStore } from "@/store/mode.store";
import { fixedStore } from "@/store/fixed.store";
import { catalogStore, load } from "@/store/catalog.store";
import { monitorById, themeList } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { ThemeTile } from "./ThemeTile";
import { Search, TriangleAlert } from "lucide-solid";
import { createMemo, createSignal, For, Show } from "solid-js";
import { Button } from "~/components/ui/button";
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
          {modeStore.mode === "fixed"
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
      <div class="min-h-0 flex-1 overflow-y-auto px-3.5 pb-4.5">
        {/* The library is the only place a theme can be picked, so a failure has to be
            readable and retryable here rather than only in a toast that has already gone. */}
        <Show when={catalogStore.status === "error"}>
          <div class="rounded-lg border border-destructive/40 p-3">
            <div class="flex items-start gap-2">
              <TriangleAlert class="mt-0.5 size-3.5 shrink-0 text-destructive" />
              <p class="min-w-0 text-[12px] font-medium">
                {t("library.loadFailed")}
              </p>
            </div>
            <p class="mt-1.5 text-[11px] leading-relaxed break-all text-muted-foreground">
              {catalogStore.error}
            </p>
            <Button
              variant="outline"
              size="sm"
              class="mt-2.5 w-full"
              onClick={() => void load()}
            >
              {t("library.reload")}
            </Button>
          </div>
        </Show>
        <Show
          when={
            catalogStore.status === "idle" || catalogStore.status === "loading"
          }
        >
          <p class="pt-1 text-[12px] text-muted-foreground">
            {t("common.loading")}
          </p>
        </Show>
        {/* Nothing installed at all, as opposed to nothing matching the search. */}
        <Show
          when={catalogStore.status === "ready" && themeList().length === 0}
        >
          <div class="rounded-lg border border-border p-3">
            <p class="text-[12px] font-medium">{t("library.empty")}</p>
            <p class="mt-1.5 text-[11px] leading-relaxed text-muted-foreground">
              {t("library.emptyDesc")}
            </p>
            <Button
              variant="outline"
              size="sm"
              class="mt-2.5 w-full"
              onClick={() => void load()}
            >
              {t("library.reload")}
            </Button>
          </div>
        </Show>
        <Show
          when={
            catalogStore.status === "ready" &&
            themeList().length > 0 &&
            list().length === 0
          }
        >
          <p class="pt-1 text-[12px] text-muted-foreground">
            {t("library.noMatch", { query: q() })}
          </p>
        </Show>
        <div class="grid grid-cols-2 gap-2.25 pt-0.5">
          <For each={list()}>
            {(theme, i) => <ThemeTile id={theme.id} index={i()} />}
          </For>
        </div>
      </div>
    </div>
  );
}

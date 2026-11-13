// Responsibility: the theme library. In fixed mode it is the picker for the current
// scope; in random mode it is the daily pool editor. One grid, one search, one filter.
import { createMemo, createSignal, For, Show } from "solid-js";
import { PageHeader } from "@/layout/PageHeader";
import { ModeToggle } from "@/layout/ModeToggle";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { t } from "@/i18n";
import { catalogStore, load as loadCatalog } from "@/store/catalog.store";
import {
  availableThemes,
  monitorById,
  themeList,
} from "@/store/catalog.selectors";
import {
  fixedStore,
  isDraftDirty,
  scopeKey,
  themeForMon,
  toggleApply,
} from "@/store/fixed.store";
import { modeStore } from "@/store/mode.store";
import {
  randomStore,
  clearAll,
  discard,
  isDirty,
  save,
  selectAll,
} from "@/store/random.store";
import { setView } from "@/store/ui.store";
import { ThemeCard } from "@/library/ThemeCard";
import { clsx } from "@/utils";
import {
  Check,
  Images,
  LoaderCircle,
  Monitor,
  Search,
  Save,
  TriangleAlert,
} from "lucide-solid";

type Filter = "all" | "installed" | "download";

export function ThemesView() {
  const [query, setQuery] = createSignal("");
  const [filter, setFilter] = createSignal<Filter>("all");

  const fixed = () => modeStore.mode === "fixed";
  const all = createMemo(() => themeList());
  const installedCount = createMemo(
    () => all().filter((theme) => !theme.needsDownload).length,
  );
  const downloadCount = createMemo(
    () => all().filter((theme) => theme.needsDownload).length,
  );

  const list = createMemo(() => {
    const q = query().trim().toLowerCase();
    return all().filter((theme) => {
      if (q && !theme.name.toLowerCase().includes(q)) return false;
      if (filter() === "installed" && theme.needsDownload) return false;
      if (filter() === "download" && !theme.needsDownload) return false;
      return true;
    });
  });

  const scopeName = () =>
    fixedStore.allUnified
      ? t("workspace.allDisplays")
      : (monitorById(fixedStore.curMon)?.name ?? t("workspace.allDisplays"));

  const selectedThemeId = () => themeForMon(scopeKey());
  const selectedTheme = () =>
    themeList().find((theme) => theme.id === selectedThemeId());
  const draftDirty = createMemo(() => isDraftDirty(scopeKey()));
  const showFixedBar = createMemo(() => fixed() && draftDirty());
  const randomDirty = createMemo(() => isDirty(randomStore));

  const applyDraft = async () => {
    await toggleApply(scopeKey());
  };

  const filters = createMemo<{ value: Filter; label: string; count: number }[]>(
    () => [
      { value: "all", label: t("themes.filterAll"), count: all().length },
      {
        value: "installed",
        label: t("themes.filterInstalled"),
        count: installedCount(),
      },
      {
        value: "download",
        label: t("themes.filterDownload"),
        count: downloadCount(),
      },
    ],
  );

  return (
    <div class="mx-auto flex min-h-full w-full max-w-[1720px] flex-col px-7 pb-28 pt-6 animate-fade-up">
      <PageHeader
        title={fixed() ? t("nav.themes") : t("workspace.pool")}
        subtitle={fixed() ? t("themes.subtitle") : t("themes.poolSubtitle")}
      >
        <ModeToggle />
        <Show when={fixed()}>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setView("displays")}
          >
            <Monitor class="size-3.5" />
            {t("workspace.manageDisplays")}
          </Button>
        </Show>
      </PageHeader>

      <Show when={fixed()}>
        <div class="mb-4 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-[12px] text-muted-foreground">
          <Monitor class="size-3.5 shrink-0 text-primary" />
          <span>{t("themes.choosingFor", { name: scopeName() })}</span>
          <span class="text-border-2">·</span>
          <span class="font-mono text-[10.5px]">
            {selectedTheme()?.name ?? "—"}
          </span>
        </div>
      </Show>

      <Show when={!fixed()}>
        <div class="mb-4 flex flex-wrap items-center gap-x-3 gap-y-2 rounded-[12px] border border-border bg-card/60 px-3.5 py-2.5 text-[11.5px] text-muted-foreground">
          <span class="font-mono text-[11px] text-foreground">
            {t("themes.poolCount", {
              selected: randomStore.selected.length,
              excluded: Math.max(
                0,
                installedCount() - randomStore.selected.length,
              ),
            })}
          </span>
          <button
            type="button"
            disabled={randomStore.selected.length === installedCount()}
            onClick={() =>
              selectAll(availableThemes().map((theme) => theme.id))
            }
            class="cursor-pointer font-medium text-primary hover:underline disabled:cursor-default disabled:opacity-40 disabled:hover:no-underline"
          >
            {t("scope.selectAll")}
          </button>
          <button
            type="button"
            disabled={randomStore.selected.length === 0}
            onClick={clearAll}
            class="cursor-pointer text-muted-foreground hover:text-foreground hover:underline disabled:cursor-default disabled:opacity-40 disabled:hover:text-muted-foreground disabled:hover:no-underline"
          >
            {t("scope.clearAll")}
          </button>
        </div>
      </Show>

      <div class="mb-4 flex flex-wrap items-center gap-3">
        <div class="relative min-w-[220px] flex-1 md:max-w-[360px]">
          <Search class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            id="theme-search"
            value={query()}
            onInput={(value) => setQuery(String(value))}
            placeholder={t("library.search")}
            class="h-9 rounded-full border-border bg-card pl-8.5 text-[12px]"
          />
        </div>

        <div class="flex flex-wrap items-center gap-1 rounded-full border border-border bg-muted/55 p-1">
          <For each={filters()}>
            {(item) => (
              <button
                type="button"
                aria-pressed={filter() === item.value}
                onClick={() => setFilter(item.value)}
                class={clsx(
                  "flex h-7 cursor-pointer items-center gap-1.5 rounded-full px-2.5 text-[11.5px] font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                  filter() === item.value
                    ? "bg-card text-foreground shadow-[0_1px_2px_rgba(0,0,0,.08)]"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                {item.label}
                <span class="font-mono text-[9.5px] opacity-60">
                  {item.count}
                </span>
              </button>
            )}
          </For>
        </div>

        <span class="ml-auto font-mono text-[10.5px] text-muted-foreground">
          {t("themes.count", { n: list().length })}
        </span>
      </div>

      <Show when={catalogStore.status === "error"}>
        <div class="mb-5 flex items-start gap-3 rounded-[12px] border border-destructive/40 bg-card p-4">
          <TriangleAlert class="mt-0.5 size-4 shrink-0 text-destructive" />
          <div class="min-w-0 flex-1">
            <p class="text-[12.5px] font-medium">{t("library.loadFailed")}</p>
            <p
              data-selectable
              class="mt-1 break-all font-mono text-[10.5px] text-muted-foreground"
            >
              {catalogStore.error}
            </p>
            <Button
              variant="outline"
              size="sm"
              class="mt-2.5"
              onClick={() => void loadCatalog()}
            >
              {t("library.reload")}
            </Button>
          </div>
        </div>
      </Show>

      <Show
        when={
          catalogStore.status === "idle" || catalogStore.status === "loading"
        }
      >
        <p class="py-10 text-center text-[12px] text-muted-foreground">
          {t("common.loading")}
        </p>
      </Show>

      <Show when={catalogStore.status === "ready" && all().length === 0}>
        <div class="flex flex-1 flex-col items-center justify-center gap-2 py-16 text-center">
          <Images class="size-9 text-muted-foreground/40" />
          <p class="font-display text-[16px] font-semibold">
            {t("themes.emptyTitle")}
          </p>
          <p class="max-w-[38ch] text-[12px] leading-relaxed text-muted-foreground">
            {t("themes.emptyDesc")}
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadCatalog()}
          >
            {t("library.reload")}
          </Button>
        </div>
      </Show>

      <Show
        when={
          catalogStore.status === "ready" &&
          all().length > 0 &&
          list().length === 0
        }
      >
        <div class="flex flex-1 flex-col items-center justify-center gap-2 py-16 text-center">
          <Search class="size-8 text-muted-foreground/40" />
          <p class="text-[12.5px] text-muted-foreground">
            {t("themes.noResults", { query: query().trim() })}
          </p>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setQuery("");
              setFilter("all");
            }}
          >
            {t("common.done")}
          </Button>
        </div>
      </Show>

      <Show when={list().length > 0}>
        <div class="grid grid-cols-2 gap-3 min-[1000px]:grid-cols-3 min-[1400px]:grid-cols-4 min-[1750px]:grid-cols-5">
          <For each={list()}>{(theme) => <ThemeCard id={theme.id} />}</For>
        </div>
      </Show>

      {/* Sticky commit bar: the library is where the choice is made, so it is also
          where the user can confirm it without going back to the stage. */}
      <Show when={showFixedBar()}>
        <div class="sticky bottom-4 z-30 mt-6 flex flex-wrap items-center gap-3 rounded-[14px] border border-border bg-popover/95 px-3.5 py-2.5 shadow-[0_16px_40px_rgba(0,0,0,.18)] backdrop-blur-md">
          <span class="flex min-w-0 items-center gap-2 text-[12px] text-muted-foreground">
            <span class="size-1.5 shrink-0 rounded-full bg-warning animate-bdot" />
            <span class="truncate">
              {t("themes.choosingFor", { name: scopeName() })}
            </span>
            <span class="truncate text-foreground">
              {selectedTheme()?.name ?? "—"}
            </span>
          </span>
          <div class="ml-auto flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setView("wallpaper")}
            >
              {t("themes.preview")}
            </Button>
            <Button
              size="sm"
              disabled={
                !selectedTheme() ||
                selectedTheme()?.needsDownload ||
                fixedStore.applying
              }
              onClick={() => void applyDraft()}
            >
              <Check class="size-3.5" />
              {t("themes.applyTo", { name: scopeName() })}
            </Button>
          </div>
        </div>
      </Show>

      <Show when={!fixed() && randomDirty()}>
        <div class="sticky bottom-4 z-30 mt-6 flex flex-wrap items-center gap-3 rounded-[14px] border border-border bg-popover/95 px-3.5 py-2.5 shadow-[0_16px_40px_rgba(0,0,0,.18)] backdrop-blur-md">
          <span class="flex items-center gap-2 text-[12px]">
            <span class="size-1.5 rounded-full bg-warning animate-bdot" />
            {t("themes.poolCount", {
              selected: randomStore.selected.length,
              excluded: Math.max(
                0,
                installedCount() - randomStore.selected.length,
              ),
            })}
          </span>
          <div class="ml-auto flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={discard}>
              {t("commit.discard")}
            </Button>
            <Button
              size="sm"
              disabled={randomStore.selected.length === 0 || randomStore.saving}
              onClick={() => void save()}
            >
              <Show
                when={randomStore.saving}
                fallback={<Save class="size-3.5" />}
              >
                <LoaderCircle class="size-3.5 animate-spin" />
              </Show>
              {randomStore.saving ? t("commit.saving") : t("themes.savePool")}
            </Button>
          </div>
        </div>
      </Show>
    </div>
  );
}

// Responsibility: random-mode stage — today's drawn theme, the sun match behind it, the
// pool it came from, and the save state of pool edits.
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import type { Wallpaper } from "@/models/types";
import { getRandomSelection, getThemeWallpapers, matchWallpaper } from "@/ipc";
import { resourceValue } from "@/lib/resource";
import { t } from "@/i18n";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { availableThemes, themeById } from "@/store/catalog.selectors";
import { randomStore } from "@/store/random.store";
import { setView } from "@/store/ui.store";
import { Button } from "@/components/ui/button";
import { clsx } from "@/utils";
import { ThemeThumbnail } from "@/scene/ThemeThumbnail";
import { PreviewPlate } from "./PreviewPlate";
import { SunPathPanel } from "./SunPathPanel";
import { RandomCommitRow } from "./RandomCommitRow";
import { CalendarDays, Images, Plus } from "lucide-solid";

export function RandomStage() {
  const [selection, { refetch: refetchSelection }] =
    createResource(getRandomSelection);
  const todayId = createMemo(() =>
    selection.error ? undefined : (selection()?.theme_id ?? undefined),
  );
  const theme = createMemo(() =>
    todayId() ? themeById(todayId()) : undefined,
  );
  const [previewIndex, setPreviewIndex] = createSignal<number | null>(null);

  const [wallpapers] = createResource(
    todayId,
    async (id): Promise<Wallpaper[]> => {
      if (!id) return [];
      const angles = await getThemeWallpapers(id);
      return angles.map((a) => ({
        index: a.index,
        solar: { altitude: a.altitude, azimuth: a.azimuth },
      }));
    },
  );

  const { position: current } = useSolarPosition({ pollMs: 60_000 });

  const [matchedEntry] = createResource(
    () => {
      const cur = resourceValue(current);
      const id = todayId();
      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  const list = createMemo(() => resourceValue(wallpapers) ?? []);
  const unique = createMemo(() => {
    const seen = new Set<number>();
    const result: Wallpaper[] = [];
    for (const w of list()) {
      if (!seen.has(w.index)) {
        seen.add(w.index);
        result.push(w);
      }
    }
    return result;
  });

  // While the matcher is re-running for a newly selected theme, its previous answer
  // describes a different theme; do not mark that frame as matched.
  const matchedIndex = createMemo(() => {
    const entry = matchedEntry.loading ? null : resourceValue(matchedEntry);
    return entry != null ? (list()[entry]?.index ?? null) : null;
  });

  createEffect(() => {
    todayId();
    setPreviewIndex(null);
  });

  // Saving a new pool can make the daemon draw a different theme; give the restart a
  // moment, then ask again instead of showing yesterday's card until the next reload.
  createEffect(() => {
    const savedAt = randomStore.savedAt;
    if (!savedAt) return;
    const timer = setTimeout(() => void refetchSelection(), 1200);
    onCleanup(() => clearTimeout(timer));
  });

  // The daemon can draw a new theme when the local day changes while this window stays
  // open. Poll slowly, and refresh immediately when the user returns to the window.
  onMount(() => {
    const refresh = () => {
      if (document.visibilityState === "visible") void refetchSelection();
    };
    const timer = setInterval(refresh, 10 * 60_000);
    document.addEventListener("visibilitychange", refresh);
    onCleanup(() => {
      clearInterval(timer);
      document.removeEventListener("visibilitychange", refresh);
    });
  });

  const active = createMemo(() => {
    const items = list();
    if (previewIndex() != null) {
      return items.find((w) => w.index === previewIndex()) ?? null;
    }
    const entry = matchedEntry.loading ? null : resourceValue(matchedEntry);
    return entry != null ? (items[entry] ?? null) : (items[0] ?? null);
  });

  const pool = createMemo(() =>
    randomStore.selected
      .map((id) => themeById(id))
      .filter((theme) => !theme.needsDownload),
  );
  const total = createMemo(() => availableThemes().length);

  return (
    <div class="grid items-start gap-5 min-[1400px]:grid-cols-[minmax(0,1fr)_360px]">
      <div class="flex min-w-0 flex-col gap-4">
        <Show
          when={theme()}
          fallback={
            <div class="flex aspect-video w-full flex-col items-center justify-center gap-3 rounded-[18px] border border-dashed border-border-2 bg-card/50 text-center">
              <CalendarDays class="size-8 text-muted-foreground/50" />
              <div class="font-display text-[16px] font-semibold">
                {t("workspace.noThemeYet")}
              </div>
              <p class="max-w-[42ch] px-4 text-[12px] leading-relaxed text-muted-foreground">
                {t("workspace.noThemeDesc")}
              </p>
              <Button size="sm" onClick={() => setView("themes")}>
                <Images class="size-3.5" />
                {t("workspace.editPool")}
              </Button>
            </div>
          }
        >
          {(today) => (
            <PreviewPlate
              themeId={today().id}
              themeName={today().name}
              scopeName={t("workspace.today")}
              wallpapers={unique()}
              wallpaper={active()}
              matchedIndex={matchedIndex()}
              onSelect={setPreviewIndex}
            />
          )}
        </Show>

        <section class="min-w-0">
          <div class="mb-2.5 flex items-baseline justify-between gap-3">
            <h2 class="text-[12px] font-semibold text-foreground">
              {t("workspace.poolPreview")}
            </h2>
            <button
              type="button"
              onClick={() => setView("themes")}
              class="cursor-pointer text-[11px] font-medium text-primary hover:underline"
            >
              {t("workspace.editPool")}
            </button>
          </div>

          <Show
            when={pool().length > 0}
            fallback={
              <button
                type="button"
                onClick={() => setView("themes")}
                class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-[14px] border border-dashed border-border-2 px-4 py-8 text-[12px] text-muted-foreground transition-colors hover:border-primary/40 hover:text-foreground"
              >
                <Plus class="size-4" />
                {t("themes.addToPool")}
              </button>
            }
          >
            <div class="grid grid-cols-2 gap-2 min-[900px]:grid-cols-3 min-[1400px]:grid-cols-4">
              <For each={pool()}>
                {(item) => (
                  <div class="group relative aspect-video overflow-hidden rounded-[10px] border border-border">
                    <ThemeThumbnail
                      themeId={item.id}
                      class="transition-transform duration-300 group-hover:scale-[1.03]"
                    />
                    <div class="pointer-events-none absolute inset-x-0 bottom-0 bg-linear-to-t from-black/80 to-transparent px-2 pb-1.5 pt-5 text-[10.5px] font-medium text-white">
                      {item.name}
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </section>
      </div>

      <aside class="flex min-w-0 flex-col gap-4">
        <section class="rounded-[16px] border border-border bg-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h2 class="text-[11px] font-semibold text-muted-foreground">
              {t("workspace.pool")}
            </h2>
            <span class="rounded-full bg-accent px-2 py-0.5 font-mono text-[9.5px] text-accent-foreground">
              24h
            </span>
          </div>

          <div class="mt-2 flex items-baseline gap-2">
            <span class="font-display text-[38px] font-semibold leading-none tracking-[-0.03em] text-warning">
              {randomStore.selected.length}
            </span>
            <span class="font-mono text-[11px] text-muted-foreground">
              / {total()}
            </span>
          </div>
          <p class="mt-1.5 text-[11px] leading-relaxed text-muted-foreground">
            {t("themes.poolSubtitle")}
          </p>

          <div class="mt-3 flex items-center justify-between border-t border-border pt-3">
            <div class="min-w-0">
              <div class="font-mono text-[9.5px] text-muted-foreground">
                {t("workspace.today")}
              </div>
              <div class="mt-1 truncate text-[12px] font-medium">
                {theme()?.name ?? t("workspace.noThemeYet")}
              </div>
            </div>
            <span
              class={clsx(
                "ml-2 size-1.5 shrink-0 rounded-full",
                theme() ? "bg-success" : "bg-muted-foreground",
              )}
            />
          </div>

          <Show when={randomStore.selected.length === 0}>
            <p class="mt-2.5 text-[10.5px] leading-relaxed text-warning">
              {t("scope.randomEmpty")}
            </p>
          </Show>
        </section>

        <Show when={resourceValue(current)}>
          {(cur) => (
            <Show when={unique().length > 0}>
              <SunPathPanel
                wallpapers={list()}
                current={cur()}
                matchedEntry={resourceValue(matchedEntry) ?? null}
                fullWidth
              />
            </Show>
          )}
        </Show>

        <RandomCommitRow />
      </aside>
    </div>
  );
}

/* ===== src/stage/FixedStage.tsx ===== */
// Responsibility: Fixed-mode stage orchestration — resolves the current theme (draft → config → first in the catalog), loads its wallpaper solar angles and the current solar position, and passes them to the pure presentation children.
import { createMemo, createResource, onCleanup, onMount, Show } from "solid-js";
import { configuredThemeId } from "@/domain/config";
import { themeById } from "@/domain/themes";
import type { Wallpaper } from "@/domain/types";
import {
  currentSolarPosition,
  getThemeWallpapers,
  matchWallpaper,
} from "@/ipc";
import { t } from "@/i18n";
import { catalogStore } from "@/store/catalog.store";
import { fixedStore, selectWallpaper } from "@/store/fixed.store";
import { settingsStore } from "@/store/settings.store";
import { ApplyRow } from "./ApplyRow";
import { Preview } from "./Preview";
import { SunPathPanel } from "./SunPathPanel";
import { WallpaperStrip } from "./WallpaperStrip";

export function FixedStage() {
  const scopeKey = createMemo(() =>
    fixedStore.allUnified ? "all" : fixedStore.curMon,
  );
  const themeId = createMemo(
    () =>
      fixedStore.monitorThemes[scopeKey()] ??
      configuredThemeId(settingsStore.config, scopeKey()) ??
      catalogStore.themes[0]?.id ??
      "",
  );
  const theme = createMemo(() => themeById(themeId()));

  const [wallpapers] = createResource(
    themeId,
    async (id): Promise<Wallpaper[]> => {
      if (!id) return [];
      const angles = await getThemeWallpapers(id);
      return angles.map((a) => ({
        index: a.index,
        solar: { altitude: a.altitude, azimuth: a.azimuth },
      }));
    },
  );

  const [current, { refetch: refetchCurrent }] = createResource(
    () => settingsStore.config?.position_source,
    (ps) => currentSolarPosition(ps),
  );

  const [matchedEntry] = createResource(
    () => {
      const cur = current();
      const id = themeId();
      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  // Refresh the solar position periodically so the match does not go stale after the app stays open a long time
  onMount(() => {
    const timer = setInterval(() => refetchCurrent(), 60_000);
    onCleanup(() => clearInterval(timer));
  });

  const list = createMemo(() => wallpapers() ?? []);
  // One image may correspond to several solar positions; the list is deduplicated by image
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
  const matchedIndex = createMemo(() => {
    const entry = matchedEntry();
    return entry != null ? (list()[entry]?.index ?? null) : null;
  });
  const active = createMemo(() => {
    const items = list();
    if (fixedStore.selIndex != null) {
      return items.find((w) => w.index === fixedStore.selIndex) ?? null;
    }
    const entry = matchedEntry();
    return entry != null ? (items[entry] ?? null) : (items[0] ?? null);
  });

  return (
    <div class="flex min-h-0 flex-1 flex-col gap-3">
      <Show when={current()}>
        {(cur) => (
          <SunPathPanel
            wallpapers={list()}
            current={cur()}
            matchedEntry={matchedEntry() ?? null}
            selIndex={fixedStore.selIndex}
            onSelect={selectWallpaper}
          />
        )}
      </Show>

      <Show
        when={list().length > 0}
        fallback={
          <div class="flex min-h-0 flex-1 items-center justify-center rounded-2xl border border-border bg-card text-center">
            <div class="space-y-1.5 px-6">
              <div class="font-display text-[14px] font-bold">
                {theme().name}
              </div>
              <div class="font-mono text-[11.5px] text-muted-foreground">
                {t("stage.notInstalled")}
              </div>
            </div>
          </div>
        }
      >
        <Show when={active()}>
          {(wp) => (
            <Preview
              themeId={themeId()}
              wallpaper={wp()}
              isMatched={wp().index === matchedIndex()}
            />
          )}
        </Show>
        <WallpaperStrip
          themeId={themeId()}
          wallpapers={unique()}
          matchedIndex={matchedIndex()}
          selIndex={fixedStore.selIndex}
          onSelect={selectWallpaper}
        />
      </Show>

      <ApplyRow />
    </div>
  );
}

// Responsibility: fixed-mode stage — resolves the current scope's theme, the sun match
// and the frame list, then lays them out as image, frames, and control deck.
import { createMemo, createResource, Show } from "solid-js";
import { themeById } from "@/store/catalog.selectors";
import { catalogStore } from "@/store/catalog.store";
import type { Wallpaper } from "@/models/types";
import { getThemeWallpapers, matchWallpaper } from "@/ipc";
import { resourceValue } from "@/lib/resource";
import { t } from "@/i18n";
import {
  fixedStore,
  scopeKey,
  selectWallpaper,
  themeForMon,
} from "@/store/fixed.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { setView } from "@/store/ui.store";
import { monitorById } from "@/store/catalog.selectors";
import { PreviewPlate } from "./PreviewPlate";
import { SunPathPanel } from "./SunPathPanel";
import { ThemeDownload } from "./ThemeDownload";
import { ThemePanel } from "./ThemePanel";
import { WallpaperRail } from "./WallpaperRail";

export function FixedStage() {
  const themeId = createMemo(() => themeForMon(scopeKey()) ?? "");
  const theme = createMemo(() => themeById(themeId()));

  const [wallpapers] = createResource(
    () => ({ id: themeId(), revision: catalogStore.installedRevision }),
    async ({ id }): Promise<Wallpaper[]> => {
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
      const id = themeId();
      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  const list = createMemo(() => resourceValue(wallpapers) ?? []);
  const installed = createMemo(() => list().length > 0);

  // One image may correspond to several solar positions; the rail is deduplicated by image.
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

  const active = createMemo(() => {
    const items = list();
    if (fixedStore.selIndex != null) {
      return items.find((w) => w.index === fixedStore.selIndex) ?? null;
    }
    const entry = matchedEntry.loading ? null : resourceValue(matchedEntry);
    return entry != null ? (items[entry] ?? null) : (items[0] ?? null);
  });

  const scopeName = () =>
    monitorById(scopeKey())?.name ?? t("workspace.allDisplays");

  return (
    <div class="grid items-start gap-5 min-[1400px]:grid-cols-[minmax(0,1fr)_360px]">
      <div class="flex min-w-0 flex-col gap-4">
        <PreviewPlate
          themeId={themeId()}
          themeName={theme().name}
          scopeName={scopeName()}
          wallpapers={unique()}
          wallpaper={active()}
          matchedIndex={matchedIndex()}
          onSelect={selectWallpaper}
        />

        <Show when={theme().needsDownload && !installed()}>
          <div class="flex items-center justify-between gap-4 rounded-[14px] border border-border bg-card px-4 py-3">
            <div class="min-w-0">
              <div class="text-[12.5px] font-medium">
                {t("stage.notInstalled")}
              </div>
              <div class="mt-0.5 font-mono text-[10.5px] text-muted-foreground">
                {theme().name}
              </div>
            </div>
            <ThemeDownload themeId={themeId()} />
          </div>
        </Show>

        <Show when={installed()}>
          <WallpaperRail
            themeId={themeId()}
            wallpapers={unique()}
            matchedIndex={matchedIndex()}
            selIndex={fixedStore.selIndex}
            onSelect={selectWallpaper}
          />
        </Show>
      </div>

      <aside class="flex min-w-0 flex-col gap-4">
        <ThemePanel
          theme={theme()}
          frameCount={unique().length}
          onChangeTheme={() => setView("themes")}
        />

        <Show when={resourceValue(current)}>
          {(cur) => (
            <SunPathPanel
              wallpapers={list()}
              current={cur()}
              matchedEntry={resourceValue(matchedEntry) ?? null}
              selIndex={fixedStore.selIndex}
              onSelect={selectWallpaper}
            />
          )}
        </Show>
      </aside>
    </div>
  );
}

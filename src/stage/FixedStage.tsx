// Responsibility: Fixed-mode stage orchestration — resolves the current theme (draft → config → first in the catalog), loads its wallpaper solar angles and the current solar position, and passes them to the pure presentation children.
import { createMemo, createResource, Show } from "solid-js";
import { themeById } from "@/store/catalog.selectors";
import type { Wallpaper } from "@/models/types";
import { getThemeWallpapers, matchWallpaper } from "@/ipc";
import { t } from "@/i18n";
import {
  fixedStore,
  isApplied,
  scopeKey,
  selectWallpaper,
  themeForMon,
} from "@/store/fixed.store";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { ApplyRow } from "./ApplyRow";
import { Preview } from "./Preview";
import { SunPathPanel } from "./SunPathPanel";
import { ThemeDownload } from "./ThemeDownload";
import { WallpaperStrip } from "./WallpaperStrip";

export function FixedStage() {
  const themeId = createMemo(() => themeForMon(scopeKey()) ?? "");
  const theme = createMemo(() => themeById(themeId()));

  const [wallpapers, { refetch: refetchWallpapers }] = createResource(
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

  // Keep refreshing the solar position so the match does not go stale while the app stays open.
  const { position: current } = useSolarPosition({ pollMs: 60_000 });

  const [matchedEntry] = createResource(
    () => {
      const cur = current();
      const id = themeId();
      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  const list = createMemo(() => wallpapers() ?? []);
  /** A theme is usable once its frames are on disk; the fallback below offers to install it. */
  const installed = createMemo(() => list().length > 0);
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
        when={installed()}
        fallback={
          <div class="flex min-h-0 flex-1 items-center justify-center rounded-2xl border border-border bg-card text-center">
            <div class="flex flex-col items-center gap-3 px-6">
              <div class="font-display text-[14px] font-bold">
                {theme().name}
              </div>
              <div class="font-mono text-[11.5px] text-muted-foreground">
                {t("stage.notInstalled")}
              </div>
              <Show when={themeId()}>
                {(id) => (
                  <ThemeDownload
                    themeId={id()}
                    onInstalled={() => void refetchWallpapers()}
                  />
                )}
              </Show>
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

      {/* Nothing to apply before the theme is installed — but a scope that is already
          applied must keep its "stop" button even if the theme files are gone. */}
      <Show when={installed() || isApplied(scopeKey())}>
        <ApplyRow />
      </Show>
    </div>
  );
}

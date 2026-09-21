/* ===== src/stage/FixedStage.tsx ===== */
// 职责：固定模式舞台编排——加载当前主题的壁纸太阳角与当前太阳位置，传给纯展示子件。
import { createMemo, createResource, Show } from "solid-js";
import type { Wallpaper } from "@/domain/types";
import { themeById } from "@/domain/themes";
import {
  currentSolarPosition,
  getThemeWallpapers,
  matchWallpaper,
} from "@/ipc";
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
      fixedStore.monitorThemes[scopeKey()] ?? catalogStore.themes[0]?.id ?? "",
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

  const [current] = createResource(
    () => settingsStore.config?.position_source,
    (ps) => currentSolarPosition(ps),
  );

  const [matchedIndex] = createResource(
    () => {
      const cur = current();
      const id = themeId();
      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  const active = createMemo(() => {
    const list = wallpapers() ?? [];
    const idx = fixedStore.selIndex ?? matchedIndex();
    return list.find((w) => w.index === idx) ?? list[0] ?? null;
  });

  return (
    <Show when={current.state === "ready" && wallpapers.state === "ready"}>
      <SunPathPanel
        wallpapers={wallpapers() ?? []}
        current={current()!}
        matchedIndex={matchedIndex() ?? null}
        selIndex={fixedStore.selIndex}
        onSelect={selectWallpaper}
      />
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
        wallpapers={wallpapers() ?? []}
        matchedIndex={matchedIndex() ?? null}
        selIndex={fixedStore.selIndex}
        onSelect={selectWallpaper}
      />
      <ApplyRow />
      <Show when={theme().id === "default"}>
        <p class="text-center font-mono text-[11px] text-muted-foreground">
          未选择主题
        </p>
      </Show>
    </Show>
  );
}

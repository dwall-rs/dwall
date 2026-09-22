/* ===== src/stage/FixedStage.tsx ===== */
// 职责：固定模式舞台编排——解析当前主题（草稿 → 配置 → 目录首个），加载其壁纸太阳角与当前太阳位置，传给纯展示子件。
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

  // 定期刷新太阳位置，避免长时间打开后匹配过期
  onMount(() => {
    const timer = setInterval(() => refetchCurrent(), 60_000);
    onCleanup(() => clearInterval(timer));
  });

  const list = createMemo(() => wallpapers() ?? []);
  // 同一张图可能对应多个太阳位置；列表按图去重展示
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

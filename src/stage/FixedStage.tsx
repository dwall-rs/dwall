/* ===== src/components/stage/FixedStage.tsx ===== */
// 职责：固定模式舞台编排——在此计算当前太阳位置与匹配壁纸，传给三个纯展示子件。
import { fixedStore, selectWallpaper } from "@/store/fixed.store";
import { themeById } from "@/domain/themes";
import { nearestWallpaper } from "@/domain/solar";
import { SunPathPanel } from "./SunPathPanel";
import { Preview } from "./Preview";
import { WallpaperStrip } from "./WallpaperStrip";
import { ApplyRow } from "./ApplyRow";
import { createMemo, createResource, Show } from "solid-js";
import { currentSolarPosition } from "@/ipc";
import { settingsStore } from "~/store/settings.store";
import { logger } from "~/utils";

const log = logger.child("FixedStage");

export function FixedStage() {
  const scopeKey = createMemo(() =>
    fixedStore.allUnified ? "all" : fixedStore.curMon,
  );
  const theme = createMemo(() =>
    themeById(fixedStore.monitorThemes[scopeKey()]),
  );
  const [current] = createResource(
    () => settingsStore.config?.position_source,
    async (ps) => {
      const current = await currentSolarPosition(ps);
      log.debug("current solar position", current);
      console.log("current solar position", current);

      return {
        solarPosition: current,
        matched: nearestWallpaper(theme().wallpapers, current),
      };
    },
  );

  const active = createMemo(() => {
    const cur = current();
    if (!cur) return null;
    const { matched } = cur;
    return (
      theme().wallpapers.find(
        (w) => w.id === (fixedStore.selWallpaperId ?? matched.id),
      ) ?? matched
    );
  });

  return (
    <Show when={current.state === "ready"}>
      <SunPathPanel
        theme={theme()}
        current={current()!.solarPosition}
        matchedId={active()!.id}
        selId={fixedStore.selWallpaperId}
        onSelect={(id) => selectWallpaper(id)}
      />
      <Preview
        type={theme().type}
        wallpaper={active()!}
        isMatched={active()!.id === current()!.matched.id}
      />
      <WallpaperStrip
        theme={theme()}
        current={current()!.solarPosition}
        matchedId={active()!.id}
        selId={fixedStore.selWallpaperId}
        onSelect={selectWallpaper}
      />
      <ApplyRow />
    </Show>
  );
}

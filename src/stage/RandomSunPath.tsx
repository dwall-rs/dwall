// Responsibility: random-mode sun track — the theme drawn for today, its frames placed on the real sun
//       path, and where the sun is now. Read-only: the pool chooses themes, not frames.
import { createMemo, createResource, Show } from "solid-js";
import type { Wallpaper } from "@/models/types";
import { getRandomSelection, getThemeWallpapers, matchWallpaper } from "@/ipc";
import { useSolarPosition } from "@/hooks/useSolarPosition";
import { themeById } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { SunPathPanel } from "./SunPathPanel";

export function RandomSunPath() {
  // The same call PoolCollage makes to badge today's tile; one invoke per mount is cheaper
  // than threading the selection through the stage as state.
  const [selection] = createResource(getRandomSelection);
  const todayId = createMemo(() =>
    selection.error ? undefined : (selection()?.themeId ?? undefined),
  );
  const theme = createMemo(() => {
    const id = todayId();
    return id ? themeById(id) : undefined;
  });

  const [wallpapers] = createResource(
    todayId,
    async (id): Promise<Wallpaper[]> => {
      if (!id) return [];

      const angles = await getThemeWallpapers(id);

      return angles.map((angle) => ({
        index: angle.index,
        solar: { altitude: angle.altitude, azimuth: angle.azimuth },
      }));
    },
  );

  // Keep the marker and the highlighted frame current while the window stays open.
  const { position: current } = useSolarPosition({ pollMs: 60_000 });

  // The daemon's own matcher, so the highlighted frame is the one it would apply rather
  // than a second guess at the same geometry.
  const [matchedEntry] = createResource(
    () => {
      const cur = current();
      const id = todayId();

      return id && cur ? { id, cur } : null;
    },
    async ({ id, cur }) => await matchWallpaper(id, cur.altitude, cur.azimuth),
  );

  const list = createMemo(() => wallpapers() ?? []);

  return (
    <Show when={list().length > 0 && current()}>
      {(cur) => (
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground">
            <span class="size-1.5 shrink-0 rounded-full bg-success" />
            <span class="truncate">
              {t("stage.randomSunPath", {
                name: theme()?.name ?? todayId() ?? "",
              })}
            </span>
          </div>
          <SunPathPanel
            wallpapers={list()}
            current={cur()}
            matchedEntry={matchedEntry() ?? null}
          />
        </div>
      )}
    </Show>
  );
}

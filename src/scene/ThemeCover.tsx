/* ===== src/scene/ThemeCover.tsx ===== */
// 职责：主题代表缩略——取目录首张缩略图（经镜像与缓存）。所有列表缩略统一走它。
import { createResource } from "solid-js";
import { getOrSaveCachedThumbnails, mirrorUrl } from "@/ipc";
import { catalogStore } from "@/store/catalog.store";
import { settingsStore } from "@/store/settings.store";
import { WallpaperImage } from "./WallpaperImage";

export function ThemeCover(props: { themeId: string; class?: string }) {
  const [src] = createResource(
    () => props.themeId,
    async (id) => {
      const theme = catalogStore.themes.find((t) => t.id === id);
      const url = theme?.thumbnails[0];
      if (!url) return null;
      const mirrored = await mirrorUrl(
        url,
        settingsStore.config?.network ?? undefined,
      );
      return await getOrSaveCachedThumbnails(id, 0, mirrored);
    },
  );
  return <WallpaperImage path={src() ?? null} class={props.class} />;
}

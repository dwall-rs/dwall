/* ===== src/scene/ThemeThumbnail.tsx ===== */
// 职责：主题缩略图（经镜像与本地缓存），按壁纸序号取。所有缩略展示统一走它。
//       注意：只加载缩略图（体积小），绝不加载原始壁纸大图。
import { createResource } from "solid-js";
import { getOrSaveCachedThumbnails, mirrorUrl } from "@/ipc";
import { catalogStore } from "@/store/catalog.store";
import { settingsStore } from "@/store/settings.store";
import { WallpaperImage } from "./WallpaperImage";

interface Props {
  themeId: string;
  index?: number;
  fit?: "cover" | "contain";
  class?: string;
}

export function ThemeThumbnail(props: Props) {
  const [src] = createResource(
    () => [props.themeId, props.index ?? 0] as const,
    async ([id, index]) => {
      const theme = catalogStore.themes.find((t) => t.id === id);
      const url = theme?.thumbnails[index];
      if (!url) return null;
      const mirrored = await mirrorUrl(
        url,
        settingsStore.config?.network ?? undefined,
      );
      return await getOrSaveCachedThumbnails(id, index, mirrored);
    },
  );
  return (
    <WallpaperImage path={src() ?? null} fit={props.fit} class={props.class} />
  );
}

// Responsibility: Theme thumbnail (through mirror + local cache), fetched by wallpaper index. Every thumbnail display goes through it.
//       Note: loads thumbnails only (small), never the full-resolution wallpaper originals.
import { createResource, type JSXElement } from "solid-js";
import { getOrSaveCachedThumbnails, mirrorUrl } from "@/ipc";
import { catalogStore } from "@/store/catalog.store";
import { settingsStore } from "@/store/settings.store";
import { WallpaperImage } from "./WallpaperImage";

interface Props {
  themeId: string;
  index?: number;
  fit?: "cover" | "contain";
  class?: string;
  fallback?: JSXElement;
}

export function ThemeThumbnail(props: Props) {
  const [src] = createResource(
    () => [props.themeId, props.index ?? 0] as const,
    async ([id, index]) => {
      const theme = catalogStore.themes.find((t) => t.id === id);
      const url = theme?.thumbnails[index];
      if (!url) return null;
      // Custom themes: thumbnails are local paths, served straight through the asset protocol — no mirroring or caching needed.
      if (theme?.source === "custom") return url;
      const mirrored = await mirrorUrl(
        url,
        settingsStore.config?.network ?? undefined,
      );
      return await getOrSaveCachedThumbnails(id, index, mirrored);
    },
  );
  return (
    <WallpaperImage
      path={src() ?? null}
      fit={props.fit}
      class={props.class}
      fallback={props.fallback}
    />
  );
}

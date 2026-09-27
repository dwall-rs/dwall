// Responsibility: Theme thumbnail (through mirror + local cache), fetched by wallpaper index. Every thumbnail display goes through it.
//       Note: loads thumbnails only (small), never the full-resolution wallpaper originals.
import {
  createEffect,
  createResource,
  createSignal,
  type JSXElement,
  onCleanup,
} from "solid-js";
import { getOrSaveCachedThumbnails, mirrorUrl } from "@/ipc";
import { catalogStore } from "@/store/catalog.store";
import { settingsStore } from "@/store/settings.store";
import { Skeleton } from "@/components/ui/skeleton";
import { WallpaperImage } from "./WallpaperImage";

/** Only surface the skeleton once loading takes longer than this, to avoid flashing on cached thumbnails. */
const SKELETON_DELAY_MS = 120;

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

  const [showSkeleton, setShowSkeleton] = createSignal(false);
  createEffect(() => {
    if (!src.loading) {
      setShowSkeleton(false);
      return;
    }
    const timer = setTimeout(() => setShowSkeleton(true), SKELETON_DELAY_MS);
    onCleanup(() => clearTimeout(timer));
  });

  return (
    <WallpaperImage
      // `src()` throws while the resource is in an error state, so check `.error` first.
      path={src.error ? null : (src() ?? null)}
      fit={props.fit}
      class={props.class}
      fallback={
        props.fallback ??
        (showSkeleton() ? (
          <Skeleton class="h-full w-full rounded-none" />
        ) : undefined)
      }
    />
  );
}

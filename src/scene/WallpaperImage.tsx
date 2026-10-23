// Responsibility: Wallpaper image (local file). Shows a placeholder when path is empty (customizable),
//               and a delayed skeleton while the image is still decoding.
import {
  createEffect,
  createSignal,
  type JSXElement,
  onCleanup,
  Show,
} from "solid-js";
import { assetUrl } from "@/ipc";
import { Skeleton } from "@/components/ui/skeleton";
import { clsx } from "@/utils";

/** Only surface the skeleton once loading takes longer than this, to avoid flashing on instant/cached images. */
const SKELETON_DELAY_MS = 120;

interface Props {
  path: string | null;
  fit?: "cover" | "contain";
  class?: string;
  fallback?: JSXElement;
}

export function WallpaperImage(props: Props) {
  const [loaded, setLoaded] = createSignal(false);
  const [errored, setErrored] = createSignal(false);
  const [showSkeleton, setShowSkeleton] = createSignal(false);

  // Reset when the source changes so a new image shows its loading state again.
  createEffect(() => {
    props.path;
    setLoaded(false);
    setErrored(false);
  });

  createEffect(() => {
    if (loaded() || errored()) {
      setShowSkeleton(false);
      return;
    }
    const timer = setTimeout(() => setShowSkeleton(true), SKELETON_DELAY_MS);
    onCleanup(() => clearTimeout(timer));
  });

  const placeholder = () =>
    props.fallback ?? (
      <div class={clsx("h-full w-full bg-card", props.class)} />
    );

  return (
    <Show when={props.path} fallback={placeholder()}>
      {(path) => (
        // A path that is on disk but will not decode — or that has gone since the catalog
        // was read — has to land on the same placeholder as a missing one: the browser's
        // broken-image icon is not a wallpaper. Reporting an error as a load also left
        // the fallback below as dead code, since it only ever ran for a null path.
        <Show when={!errored()} fallback={placeholder()}>
          <div class="relative h-full w-full">
            <img
              src={assetUrl(path())}
              alt=""
              onLoad={() => setLoaded(true)}
              onError={() => setErrored(true)}
              class={clsx(
                "block h-full w-full",
                props.fit === "contain" ? "object-contain" : "object-cover",
                props.class,
              )}
            />
            <Show when={showSkeleton()}>
              <Skeleton class="absolute inset-0 h-full w-full rounded-none" />
            </Show>
          </div>
        </Show>
      )}
    </Show>
  );
}

import {
  createEffect,
  createSignal,
  mergeProps,
  onCleanup,
  Show,
} from "solid-js";
import type { JSX } from "solid-js";

import { convertFileSrc } from "@tauri-apps/api/core";

import { getOrSaveCachedThumbnails } from "~/commands";

import type { ImageProps as ThemeImageProps } from "./Image.types";

import { clsx } from "~/utils";
import { Skeleton } from "../skeleton";
import { ImageOff } from "lucide-solid";

const ThemeImage = (props: ThemeImageProps) => {
  let imageRef: HTMLImageElement | undefined;

  const [loaded, setLoaded] = createSignal(false);
  const [error, setError] = createSignal<Error | null>(null);
  const [retryAttempts, setRetryAttempts] = createSignal(0);
  const merged = mergeProps({ retryCount: 3 }, props);

  const [resolvedSrc, setResolvedSrc] = createSignal<string | null>(null);

  const handleLoad = () => {
    if (imageRef?.src) {
      setLoaded(true);
      setError(null);

      const { top, bottom, left, right } = imageRef.getBoundingClientRect();
      props.onLoad?.({
        width: imageRef.naturalWidth,
        height: imageRef.naturalHeight,
        top,
        bottom,
        left,
        right,
      });
    }
  };

  const handleError = () => {
    const currentAttempts = retryAttempts();
    if (currentAttempts < merged.retryCount) {
      setRetryAttempts(currentAttempts + 1);
      // Retry loading
      loadCachedImage();
    } else {
      const err = new Error(
        `Failed to load image after ${merged.retryCount} attempts`,
      );
      setError(err);
      props.onError?.(err);

      if (props.fallbackSrc) {
        setResolvedSrc(props.fallbackSrc);
      }
    }
  };

  const loadCachedImage = async () => {
    try {
      const path = await getOrSaveCachedThumbnails(
        props.themeID,
        props.serialNumber,
        props.src,
      );
      const src = convertFileSrc(path);
      setResolvedSrc(src);
    } catch (err) {
      console.error(err);
      handleError();
    }
  };

  const setupLazyLoading = () => {
    if (!imageRef) return;

    const observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && !resolvedSrc()) {
          loadCachedImage();
          observer.unobserve(entry.target);
        }
      }
    });

    observer.observe(imageRef);

    return () => observer.disconnect();
  };

  createEffect(() => {
    const cleanup = setupLazyLoading();
    if (cleanup) onCleanup(cleanup);
  });

  const createContainerStyle = (): JSX.CSSProperties => ({
    width: props.width ? `${props.width}px` : undefined,
    height: props.height ? `${props.height}px` : undefined,
  });

  return (
    <>
      <Show when={!loaded() && !error()}>
        <Show
          when={props.skeleton}
          fallback={<Skeleton class="flex-1 w-full h-full" />}
        >
          {props.skeleton}
        </Show>
      </Show>

      <div
        class={clsx(
          "relative inline-flex items-center justify-center",
          !loaded() && !error() ? "w-0 h-0" : "w-full h-full",
        )}
        style={createContainerStyle()}
      >
        <img
          ref={imageRef}
          alt={props.alt}
          src={resolvedSrc() || undefined}
          onLoad={handleLoad}
          onError={handleError}
          width={props.width}
          class={clsx(loaded() ? "visible" : "invisible", props.class)}
        />

        <Show when={error() && !props.fallbackSrc}>
          <div>
            <ImageOff />
          </div>
        </Show>
      </div>
    </>
  );
};

export default ThemeImage;

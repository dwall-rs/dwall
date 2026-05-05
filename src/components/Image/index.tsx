import {
  children,
  createEffect,
  createSignal,
  mergeProps,
  onCleanup,
} from "solid-js";
import type { JSX } from "solid-js";

import { convertFileSrc } from "@tauri-apps/api/core";

import { Spinner } from "../spinner";

import { getOrSaveCachedThumbnails } from "~/commands";

import type { ImageProps as ThemeImageProps } from "./Image.types";

import { clsx } from "~/utils";

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

  const renderImageContent = children(() => (
    <>
      {!loaded() && !error() && (
        <div class="absolute top-1/2 left-1/2 -translate-1/2">
          <Spinner />
        </div>
      )}
      <img
        ref={imageRef}
        alt={props.alt}
        src={resolvedSrc() || undefined}
        onLoad={handleLoad}
        onError={handleError}
        width={props.width}
        class={clsx(loaded() ? "visible" : "invisible", props.class)}
      />
      {error() && !props.fallbackSrc && <div>Failed to load image</div>}
    </>
  ));

  const createContainerStyle = (): JSX.CSSProperties => ({
    width: props.width ? `${props.width}px` : undefined,
    height: props.height ? `${props.height}px` : undefined,
  });

  return (
    <div
      class={clsx("relative inline-flex items-center justify-center")}
      style={createContainerStyle()}
    >
      {renderImageContent()}
    </div>
  );
};

export default ThemeImage;

import { convertFileSrc } from "@tauri-apps/api/core";
import {
  children,
  createEffect,
  createSignal,
  mergeProps,
  onCleanup,
} from "solid-js";
import { getOrSaveCachedThumbnails } from "~/commands";
import { LazySpinner } from "~/lazy";

interface ImageData {
  width: number;
  height: number;
}

interface ImageProps {
  themeID: string;
  serialNumber: number;
  src: string;
  alt?: string;
  width?: number;
  height?: number;
  class?: string;
  onLoad?: (data: ImageData) => void;
  onError?: (error: Error) => void;
  fallbackSrc?: string;
  retryCount?: number;
}

const Image = (props: ImageProps) => {
  let imageRef: HTMLImageElement | undefined;
  const [loaded, setLoaded] = createSignal(false);
  const [error, setError] = createSignal<Error | null>(null);
  const [retryAttempts, setRetryAttempts] = createSignal(0);
  const merged = mergeProps({ retryCount: 3 }, props);

  const [currentSrc, setCurrentSrc] = createSignal<string | null>(null);

  const handleLoad = () => {
    if (imageRef?.src) {
      setLoaded(true);
      setError(null);
      props.onLoad?.({
        width: imageRef.naturalWidth,
        height: imageRef.naturalHeight,
      });
    }
  };

  const handleError = () => {
    const currentAttempts = retryAttempts();
    if (currentAttempts < merged.retryCount) {
      setRetryAttempts(currentAttempts + 1);
      // 重试加载
      loadImage();
    } else {
      const err = new Error(
        `Failed to load image after ${merged.retryCount} attempts`,
      );
      setError(err);
      props.onError?.(err);

      if (props.fallbackSrc) {
        setCurrentSrc(props.fallbackSrc);
      }
    }
  };

  const loadImage = async () => {
    try {
      const path = await getOrSaveCachedThumbnails(
        props.themeID,
        props.serialNumber,
        props.src,
      );
      const src = convertFileSrc(path);
      setCurrentSrc(src);
    } catch (err) {
      handleError();
    }
  };

  createEffect(() => {
    if (!imageRef) return;

    const observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && !currentSrc()) {
          loadImage();
          observer.unobserve(entry.target);
        }
      }
    });

    observer.observe(imageRef);

    onCleanup(() => {
      observer.disconnect();
    });
  });

  const resolved = children(() => (
    <>
      {!loaded() && !error() && (
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
          }}
        >
          <LazySpinner />
        </div>
      )}
      <img
        ref={imageRef}
        alt={props.alt}
        src={currentSrc() || undefined}
        onLoad={handleLoad}
        onError={handleError}
        width={props.width}
        style={{ visibility: loaded() ? "visible" : "hidden" }}
      />
      {error() && !props.fallbackSrc && <div>Failed to load image</div>}
    </>
  ));

  return (
    <div
      class={props.class}
      style={{
        width: props.width && `${props.width}px`,
        height: props.height && `${props.height}px`,
        position: "relative",
        display: "inline-flex",
        "align-items": "center",
        "justify-content": "center",
      }}
    >
      {resolved()}
    </div>
  );
};

export default Image;

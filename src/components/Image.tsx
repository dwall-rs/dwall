import { convertFileSrc } from "@tauri-apps/api/core";
import { children, createSignal, onCleanup } from "solid-js";
import { getOrSaveCachedImage } from "~/commands";
import { LazySpinner } from "~/lazy";

interface ImageData {
  width: number;
  height: number;
}

interface ImageProps {
  ref?: HTMLImageElement;
  themeID: string;
  serialNumber: number;
  src: string;
  alt?: string;
  width?: number;
  height?: number;
  class?: string;
  onLoad?: (data: ImageData) => void;
}

const Image = (props: ImageProps) => {
  const [loaded, setLoaded] = createSignal(false);
  const [isSrcSet, setIsSrcSet] = createSignal(false);

  const handleLoad = () => {
    const img = props.ref;
    if (img) {
      setLoaded(true);
      props.onLoad?.({
        width: img.naturalWidth,
        height: img.naturalHeight,
      });
    }
  };

  const handleError = () => {
    console.error("Image failed to load");
  };

  const observerCallback: IntersectionObserverCallback = (entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting && props.ref && !isSrcSet()) {
        getOrSaveCachedImage(props.themeID, props.serialNumber, props.src).then(
          (path) => {
            const src = convertFileSrc(path);
            props.ref!.src = src;
            setIsSrcSet(true);
            observer.unobserve(props.ref!);
          },
        );
      }
    }
  };

  const observer = new IntersectionObserver(observerCallback);

  onCleanup(() => {
    if (props.ref) {
      observer.unobserve(props.ref);
    }
  });

  const resolved = children(() => (
    <>
      {!loaded() && (
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
        ref={(el) => {
          props.ref = el;
          if (el) {
            observer.observe(el);
          }
        }}
        alt={props.alt}
        onLoad={handleLoad}
        onError={handleError}
        width={props.width}
        style={{ visibility: loaded() ? "visible" : "hidden" }}
      />
    </>
  ));

  return (
    <div
      class={props.class}
      style={{
        width: props.width ? `${props.width}px` : "auto",
        height: props.height ? `${props.height}px` : "auto",
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

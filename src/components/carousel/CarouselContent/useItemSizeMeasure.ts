import { type Accessor, onCleanup, onMount } from "solid-js";
import type { Orientation } from "../Carousel/Carousel.types";

export function useItemSizeMeasure(
  getViewport: () => HTMLElement | undefined,
  getTrack: () => HTMLElement | undefined,
  orientation: Accessor<Orientation>,
  setItemSize: (size: number) => void,
  setViewportSize: (size: number) => void,
) {
  onMount(() => {
    const viewport = getViewport();
    const track = getTrack();
    if (!viewport || !track) return;

    const measure = () => {
      // item 尺寸：track 的第一个子元素
      const firstItem = track.firstElementChild as HTMLElement | null;
      if (firstItem) {
        setItemSize(
          orientation() === "horizontal"
            ? firstItem.offsetWidth
            : firstItem.offsetHeight,
        );
      }
      // viewport 尺寸：overflow:hidden 容器
      setViewportSize(
        orientation() === "horizontal"
          ? viewport.offsetWidth
          : viewport.offsetHeight,
      );
    };

    measure();

    // 同一个 observer 同时监听两个元素，回调合并触发
    const ro = new ResizeObserver(measure);
    ro.observe(viewport);
    ro.observe(track);
    onCleanup(() => ro.disconnect());
  });
}

import { createEffect, createSignal } from "solid-js";
import type { CarouselProps } from "./Carousel.types";

export function useCarousel(
  opts: Pick<
    CarouselProps,
    "autoPlay" | "autoPlayInterval" | "loop" | "onChange" | "orientation" | "id"
  >,
) {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [itemCount, setItemCount] = createSignal(0);
  const [itemSize, setItemSize] = createSignal(0);
  const [viewportSize, setViewportSize] = createSignal(0);

  const loop = () => opts.loop ?? false;
  const orientation = () => opts.orientation ?? "horizontal";

  /**
   * 视口内能完整显示的 item 数量。
   * itemSize > 0 时才能计算，否则回退到 1（保守估计）。
   * 使用 Math.round 而非 Math.floor：
   *   浮点测量误差（如 333.33px × 3 ≠ 1000px）可能导致
   *   floor 少算一个，round 能消除亚像素误差。
   */
  const visibleCount = () => {
    const iSize = itemSize();
    if (iSize <= 0) return 1;
    return Math.round(viewportSize() / iSize);
  };

  const canScrollPrev = () => loop() || currentIndex() > 0;

  const canScrollNext = () =>
    loop() || currentIndex() + visibleCount() < itemCount();

  const scrollTo = (index: number) => {
    const count = itemCount();
    if (count === 0) return;

    let nextIndex = -1;
    if (loop()) {
      nextIndex = ((index % count) + count) % count;
    } else {
      nextIndex = Math.max(0, Math.min(index, count - 1));
    }
    setCurrentIndex(nextIndex);
  };

  const scrollPrev = () => scrollTo(currentIndex() - 1);
  const scrollNext = () => scrollTo(currentIndex() + 1);

  const registerItem = () => {
    setItemCount((n) => n + 1);
    return () => setItemCount((n) => n - 1);
  };

  createEffect(() => {
    opts.onChange?.(itemCount(), currentIndex());
  });

  createEffect(() => {
    if (opts.id) {
      setCurrentIndex(0);
    }
  });

  return {
    currentIndex,
    itemCount,
    itemSize,
    setItemSize,
    viewportSize,
    setViewportSize,
    orientation,
    loop,
    canScrollPrev,
    canScrollNext,
    scrollPrev,
    scrollNext,
    scrollTo,
    registerItem,
  };
}

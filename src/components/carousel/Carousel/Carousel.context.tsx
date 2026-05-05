import { type Accessor, createContext, useContext } from "solid-js";
import type { Orientation } from "./Carousel.types";

interface CarouselContextValue {
  currentIndex: Accessor<number>;
  itemCount: Accessor<number>;
  orientation: Accessor<Orientation>;
  loop: Accessor<boolean>;
  canScrollPrev: Accessor<boolean>;
  canScrollNext: Accessor<boolean>;
  scrollPrev: () => void;
  scrollNext: () => void;
  scrollTo: (index: number) => void;
  registerItem: () => () => void;
  itemSize: Accessor<number>;
  setItemSize: (size: number) => void;
  /**
   * viewport（overflow:hidden 容器）的实测像素尺寸。
   * 由 CarouselContent 的 ResizeObserver 写入，用于派生 visibleCount。
   */
  viewportSize: Accessor<number>;
  setViewportSize: (size: number) => void;
}

export const CarouselContext = createContext<CarouselContextValue>();

export function useCarouselContext() {
  const context = useContext(CarouselContext);
  if (!context) {
    throw new Error("useCarouselContext must be used within a <Carousel />");
  }
  return context;
}

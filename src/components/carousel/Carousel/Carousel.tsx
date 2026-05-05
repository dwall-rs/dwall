import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import { CarouselContext } from "./Carousel.context";
import type { CarouselProps } from "./Carousel.types";
import { useAutoPlay } from "./useAutoPlay";
import { useCarousel } from "./useCarousel";
import { useKeyboardNavigation } from "./useKeyboardNavigation";

export const Carousel = (props: CarouselProps) => {
  const [local, options, rest] = splitProps(
    props,
    ["class", "aria-label"],
    ["loop", "orientation", "autoPlay", "autoPlayInterval", "onChange", "id"],
  );

  const state = useCarousel(options);
  let rootRef: HTMLElement | undefined;

  useAutoPlay(
    options.autoPlay ?? false,
    options.autoPlayInterval ?? 3000,
    state.scrollNext,
  );

  useKeyboardNavigation(
    rootRef,
    state.orientation,
    state.scrollPrev,
    state.scrollNext,
  );

  return (
    <CarouselContext.Provider value={state}>
      <section
        ref={rootRef}
        data-slot="carousel"
        aria-label={local["aria-label"] ?? "Carousel"}
        aria-roledescription="carousel"
        class={clsx("relative", local.class)}
        {...rest}
      />
    </CarouselContext.Provider>
  );
};

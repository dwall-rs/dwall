import { useCarouselContext } from "../Carousel";
import type { CarouselDotsProps } from "./CarouselDots.types";

export const CarouselDots = (props: CarouselDotsProps) => {
  const { currentIndex, itemCount, scrollTo } = useCarouselContext();

  return (
    <div
      role="tablist"
      aria-label="Slide navigation"
      class={[
        "flex items-center justify-center gap-1.5 pt-3",
        props.class ?? "",
      ].join(" ")}
    >
      {Array.from({ length: itemCount() }).map((_, i) => (
        <button
          type="button"
          role="tab"
          aria-selected={i === currentIndex()}
          aria-label={`Go to slide ${i + 1}`}
          onClick={() => scrollTo(i)}
          class={[
            "h-2 rounded-full transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2",
            i === currentIndex()
              ? "w-6 bg-neutral-900"
              : "w-2 bg-neutral-300 hover:bg-neutral-400",
          ].join(" ")}
        />
      ))}
    </div>
  );
};

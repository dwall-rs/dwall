import { ChevronLeft } from "lucide-solid";
import { mergeProps, splitProps } from "solid-js";
import { Button } from "~/components/button";
import { clsx } from "~/utils";
import { useCarouselContext } from "../Carousel";
import type { CarouselPreviousProps } from "./CarouselPrevious.types";

export const CarouselPrevious = (props: CarouselPreviousProps) => {
  const { scrollPrev, canScrollPrev, orientation } = useCarouselContext();
  const merged = mergeProps({ variant: "outline", size: "sm" } as const, props);
  const [local, others] = splitProps(merged, ["class", "classList"]);

  return (
    <Button
      data-slot="carousel-previous"
      class={clsx(
        "absolute touch-manipulation rounded-full",
        orientation() === "horizontal" &&
          "active:data-[slot=carousel-previous]:translate-y-[calc(-50%+1px)]", // 修复shadcn中存在的bug，active状态会出现异常位移
        orientation() === "horizontal"
          ? "top-1/2 -left-12 -translate-y-1/2"
          : "-top-12 left-1/2 -translate-x-1/2 rotate-90",
        local.class,
      )}
      disabled={!canScrollPrev()}
      onClick={scrollPrev}
      aria-label="Previous slide"
      icon={{
        icon: <ChevronLeft class="cn-rtl-flip" />,
        ariaLabel: "Previous slide",
      }}
      {...others}
    />
  );
};

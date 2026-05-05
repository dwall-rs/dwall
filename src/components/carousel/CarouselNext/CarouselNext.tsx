import { ChevronRight } from "lucide-solid";
import { mergeProps, splitProps } from "solid-js";
import { Button } from "~/components/button";
import { clsx } from "~/utils";
import { useCarouselContext } from "../Carousel";
import type { CarouselNextProps } from "./CarouselNext.types";

export const CarouselNext = (props: CarouselNextProps) => {
  const { scrollNext, canScrollNext, orientation } = useCarouselContext();
  const merged = mergeProps({ variant: "outline", size: "sm" } as const, props);
  const [local, others] = splitProps(merged, ["class", "classList"]);

  return (
    <Button
      data-slot="carousel-next"
      class={clsx(
        "absolute touch-manipulation rounded-full",
        orientation() === "horizontal" &&
          "active:data-[slot=carousel-next]:translate-y-[calc(-50%+1px)]", // 修复shadcn中存在的bug，active状态会出现异常位移
        orientation() === "horizontal"
          ? "top-1/2 -right-12 -translate-y-1/2"
          : "-bottom-12 left-1/2 -translate-x-1/2 rotate-90",
        local.class,
      )}
      aria-label="Next slide"
      disabled={!canScrollNext()}
      onClick={scrollNext}
      icon={{
        icon: <ChevronRight class="cn-rtl-flip" />,
        ariaLabel: "Next slide",
      }}
      {...others}
    />
  );
};

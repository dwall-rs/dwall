import { onCleanup, onMount } from "solid-js";
import { clsx } from "~/utils";
import { useCarouselContext } from "../Carousel";
import type { CarouselItemProps } from "./CarouselItem.types";

export const CarouselItem = (props: CarouselItemProps) => {
  const { currentIndex, registerItem, orientation } = useCarouselContext();

  onMount(() => {
    const unregister = registerItem();
    // 记录注册顺序（简化：由父层传入 index）
    onCleanup(unregister);
  });

  const isActive = () =>
    props.index !== undefined ? props.index === currentIndex() : false;

  return (
    <fieldset
      aria-roledescription="slide"
      data-slot="carousel-item"
      aria-label={
        props.index !== undefined ? `Slide ${props.index + 1}` : "Slide"
      }
      aria-hidden={!isActive()}
      class={clsx(
        "min-w-0 shrink-0 grow-0 basis-full",
        orientation() === "horizontal" ? "pl-4" : "pt-4",
        props.class,
      )}
    >
      {props.children}
    </fieldset>
  );
};

import type { JSX } from "solid-js";
import { clsx } from "~/utils";
import { useScrollEdges } from "~/hooks/useScrollEdges";
import { ScrollArrowButton } from "./ScrollArrowButton";
import { useHoverScroll } from "./useHoverScroll";
import type { ScrollArrowsProps } from "./ScrollArrows.types";

export function ScrollArrows(props: ScrollArrowsProps): JSX.Element {
  const { canScrollUp, canScrollDown } = useScrollEdges(() => props.target());

  useHoverScroll({
    target: () => props.target(),
    canScrollUp,
    canScrollDown,
    enabled: () => props.hoverScroll ?? true,
  });

  return (
    <>
      <ScrollArrowButton
        direction="up"
        visible={canScrollUp()}
        class={clsx(props.class, props.upClass)}
      />
      <ScrollArrowButton
        direction="down"
        visible={canScrollDown()}
        class={clsx(props.class, props.downClass)}
      />
    </>
  );
}

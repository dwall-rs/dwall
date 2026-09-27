import type { JSX } from "solid-js";
import { clsx } from "~/utils";
import { useScrollEdges } from "~/hooks/useScrollEdges";
import { ScrollArrowButton } from "./ScrollArrowButton";
import type { ScrollArrowsProps } from "./ScrollArrows.types";

/**
 * Generic scroll indicators for lists/menu overlays: with the native scrollbar
 * hidden, arrows are overlaid on the top/bottom edge to indicate more content
 * in that direction; hovering or holding an arrow scrolls continuously, and
 * arrows hide automatically at the boundary. Place it inside the scrolling
 * container (the positioned ancestor); the container must itself be positioned.
 */
export function ScrollArrows(props: ScrollArrowsProps): JSX.Element {
  const { canScrollUp, canScrollDown } = useScrollEdges(() => props.target());

  return (
    <>
      <ScrollArrowButton
        direction="up"
        visible={canScrollUp()}
        target={props.target}
        class={clsx(props.class, props.upClass)}
      />
      <ScrollArrowButton
        direction="down"
        visible={canScrollDown()}
        target={props.target}
        class={clsx(props.class, props.downClass)}
      />
    </>
  );
}

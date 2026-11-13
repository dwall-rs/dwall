import type { Accessor } from "solid-js";

export interface ScrollArrowsProps {
  target: Accessor<HTMLElement | undefined>;

  hoverScroll?: boolean;

  class?: string;

  upClass?: string;

  downClass?: string;
}

export interface ScrollArrowButtonProps {
  direction: "up" | "down";
  visible: boolean;
  class?: string;
}

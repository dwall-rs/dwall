import type { Accessor } from "solid-js";

export interface ScrollArrowsProps {
  /** Scrolling container accessor; the component derives edge states from it */
  target: Accessor<HTMLElement | undefined>;
  /** Extra class shared by both arrows */
  class?: string;
  /** Extra class for the top arrow (border radius, etc.) */
  upClass?: string;
  /** Extra class for the bottom arrow (border radius, etc.) */
  downClass?: string;
}

export interface ScrollArrowButtonProps {
  direction: "up" | "down";
  visible: boolean;
  target: Accessor<HTMLElement | undefined>;
  class?: string;
}

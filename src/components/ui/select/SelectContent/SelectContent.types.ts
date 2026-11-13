import type { ParentProps } from "solid-js";
import type { Alignment, Placement, Side } from "~/lib";

export type SelectSide = Side | "inline-start" | "inline-end";

export type SelectAlign = Alignment | "center";

export interface SelectContentProps extends ParentProps {
  side?: SelectSide;

  align?: SelectAlign;

  sideOffset?: number;

  alignOffset?: number;

  alignItemWithTrigger?: boolean;

  collisionPadding?: number;

  placement?: Placement;
  class?: string;

  [key: string]: any;
}

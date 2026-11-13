import type { Alignment, Side } from "~/lib";
import type { BaseProps, PolymorphicProps } from "~/types";

export type TooltipSide = Side | "inline-start" | "inline-end";

interface BaseTooltipContentProps extends BaseProps {
  side?: TooltipSide;

  align?: Alignment | "center";

  sideOffset?: number;

  alignOffset?: number;

  collisionPadding?: number;
}

export type TooltipContentProps = PolymorphicProps<
  "div",
  BaseTooltipContentProps,
  false
>;

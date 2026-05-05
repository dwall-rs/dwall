import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseTooltipContentProps extends BaseProps {
  side?: "top" | "bottom" | "left" | "right";
  align?: "start" | "center" | "end";
  alignOffset?: number;
  sideOffset?: number;
}

export type TooltipContentProps = PolymorphicProps<
  "div",
  BaseTooltipContentProps,
  false
>;

import type { JSX } from "solid-js";
import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseSeparatorProps extends BaseProps {
  orientation?: "horizontal" | "vertical";
  style?: JSX.CSSProperties;
}

export type SeparatorProps = PolymorphicProps<"div", BaseSeparatorProps, false>;

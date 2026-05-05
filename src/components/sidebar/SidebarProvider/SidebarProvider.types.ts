import type { JSX } from "solid-js";
import type { BaseProps, PolymorphicProps } from "~/types";

export type SidebarProviderProps = PolymorphicProps<
  "div",
  BaseProps & {
    defaultOpen?: boolean;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    style?: JSX.CSSProperties;
  },
  false
>;

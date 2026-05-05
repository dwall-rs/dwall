import type { BaseProps, PolymorphicProps } from "~/types";

export type SidebarProps = PolymorphicProps<
  "div",
  BaseProps & {
    side?: "left" | "right";
    variant?: "sidebar" | "floating" | "inset";
    collapsible?: "offcanvas" | "icon" | "none";
  },
  false
>;

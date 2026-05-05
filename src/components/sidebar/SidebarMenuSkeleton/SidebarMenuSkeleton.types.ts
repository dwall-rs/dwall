import type { BaseProps, PolymorphicProps } from "~/types";

export type SidebarMenuSkeletonProps = PolymorphicProps<
  "div",
  BaseProps & {
    showIcon?: boolean;
  },
  false
>;

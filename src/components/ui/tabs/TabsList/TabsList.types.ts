import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { tabsListVariants } from "./TabsList.styles";

export type TabsListProps = PolymorphicProps<
  "div",
  BaseProps &
    VariantProps<typeof tabsListVariants> & {
      /** 聚焦 tab 时是否同时激活,默认 false(与 base-ui 一致) */
      activateOnFocus?: boolean;
    },
  false
>;

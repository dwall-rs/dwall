import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { tabsListVariants } from "./TabsList.styles";

export type TabsListProps = PolymorphicProps<
  "div",
  BaseProps &
    VariantProps<typeof tabsListVariants> & {
      activateOnFocus?: boolean;
    },
  false
>;

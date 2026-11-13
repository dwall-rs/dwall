import type { BaseProps, PolymorphicProps } from "~/types";

export type TabsValue = string | number;

interface BaseTabsProps extends BaseProps {
  defaultValue?: TabsValue;

  value?: TabsValue;

  onValueChange?: (value: TabsValue) => void;

  orientation?: "horizontal" | "vertical";

  loop?: boolean;
}

export type TabsProps = PolymorphicProps<"div", BaseTabsProps, false>;

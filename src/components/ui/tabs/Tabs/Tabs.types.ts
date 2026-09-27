import type { BaseProps, PolymorphicProps } from "~/types";

export type TabsValue = string | number;

interface BaseTabsProps extends BaseProps {
  /** Uncontrolled mode: initial activated value */
  defaultValue?: TabsValue;
  /** Controlled mode: activated value (once provided, the component stops managing state) */
  value?: TabsValue;
  /** Value change callback (fires in both controlled and uncontrolled modes) */
  onValueChange?: (value: TabsValue) => void;
  /** Layout orientation, defaults to "horizontal" */
  orientation?: "horizontal" | "vertical";
  /** Whether arrow keys loop, defaults to true */
  loop?: boolean;
}

export type TabsProps = PolymorphicProps<"div", BaseTabsProps, false>;

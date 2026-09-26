import type { BaseProps, PolymorphicProps } from "~/types";

export type TabsValue = string | number;

interface BaseTabsProps extends BaseProps {
  /** 非受控模式:初始激活值 */
  defaultValue?: TabsValue;
  /** 受控模式:激活值(提供后组件不再自管状态) */
  value?: TabsValue;
  /** 值变化回调(受控与非受控都会触发) */
  onValueChange?: (value: TabsValue) => void;
  /** 布局方向,默认 "horizontal" */
  orientation?: "horizontal" | "vertical";
  /** 方向键是否循环,默认 true */
  loop?: boolean;
}

export type TabsProps = PolymorphicProps<"div", BaseTabsProps, false>;

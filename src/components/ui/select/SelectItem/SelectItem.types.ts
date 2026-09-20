import type { ParentProps } from "solid-js";

export interface SelectItemProps extends ParentProps {
  value: string;
  /** 不传的话，会尝试用 children 是字符串时的内容当 label，否则用 value 本身 */
  label?: string;
  disabled?: boolean;
  class?: string;
  [key: string]: any;
}

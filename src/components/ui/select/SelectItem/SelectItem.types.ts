import type { ParentProps } from "solid-js";
import type { SelectOptionValue } from "../Select/Select.types";

export interface SelectItemProps<
  T extends SelectOptionValue = SelectOptionValue,
> extends ParentProps {
  value: T;
  /** When omitted, the string content of children is used as the label if possible, otherwise the value itself */
  label?: string;
  disabled?: boolean;
  class?: string;
  [key: string]: any;
}

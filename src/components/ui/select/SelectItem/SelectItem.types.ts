import type { ParentProps } from "solid-js";
import type { SelectOptionValue } from "../Select/Select.types";

export interface SelectItemProps<
  T extends SelectOptionValue = SelectOptionValue,
> extends ParentProps {
  value: T;

  label?: string;
  disabled?: boolean;
  class?: string;
  [key: string]: any;
}

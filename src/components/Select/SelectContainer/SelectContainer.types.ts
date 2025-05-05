import type { JSX } from "solid-js";

export type SelectContainerProps = {
  children: JSX.Element;
  label?: string;
  required?: boolean;
  warning?: string;
  labelId?: string;
};

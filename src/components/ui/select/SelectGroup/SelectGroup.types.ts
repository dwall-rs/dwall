import type { ParentProps } from "solid-js";
import type { BaseProps } from "~/types";

export type SelectGroupProps = BaseProps &
  ParentProps & { [key: `data-${string}`]: unknown };

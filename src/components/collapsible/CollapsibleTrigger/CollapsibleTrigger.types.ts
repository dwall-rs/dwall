import type { ParentProps } from "solid-js";
import type { BaseButtonProps } from "~/components/button";

export type CollapsibleTriggerProps = ParentProps<
  Omit<BaseButtonProps, "onClick">
>;

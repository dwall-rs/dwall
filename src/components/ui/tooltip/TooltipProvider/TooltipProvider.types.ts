import type { Accessor, ParentProps } from "solid-js";

export interface TooltipProviderProps extends ParentProps {
  delay?: number;

  closeDelay?: number;
}

export interface TooltipProviderContextValue {
  delay: Accessor<number>;
  closeDelay: Accessor<number>;
}

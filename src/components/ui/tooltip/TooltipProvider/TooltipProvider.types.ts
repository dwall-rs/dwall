import type { Accessor, ParentProps } from "solid-js";

export interface TooltipProviderProps extends ParentProps {
  /** Delay before opening on hover, default 0 (consistent with shadcn Base UI). */
  delay?: number;
  /** Delay before closing after hover leaves, default 150. */
  closeDelay?: number;
}

export interface TooltipProviderContextValue {
  delay: Accessor<number>;
  closeDelay: Accessor<number>;
}

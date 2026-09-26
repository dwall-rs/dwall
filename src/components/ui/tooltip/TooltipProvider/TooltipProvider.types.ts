import type { Accessor, ParentProps } from "solid-js";

export interface TooltipProviderProps extends ParentProps {
  /** hover 后延迟多久打开，默认 0（与 shadcn Base UI 保持一致） */
  delay?: number;
  /** hover 移出后延迟多久关闭，默认 150 */
  closeDelay?: number;
}

export interface TooltipProviderContextValue {
  delay: Accessor<number>;
  closeDelay: Accessor<number>;
}

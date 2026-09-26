import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "solid-js";
import type { toggleVariants } from "./Toggle.styles";

export interface ToggleProps
  extends ComponentProps<"button">,
    VariantProps<typeof toggleVariants> {
  /** 受控按下状态；不传则内部自管理 */
  pressed?: boolean;
  /** 非受控模式下的初始按下状态 */
  defaultPressed?: boolean;
  onPressedChange?: (pressed: boolean) => void;
  /** 在 ToggleGroup 中使用的唯一值 */
  value?: string;
}

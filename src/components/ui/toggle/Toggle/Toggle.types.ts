import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "solid-js";
import type { toggleVariants } from "./Toggle.styles";

export interface ToggleProps
  extends ComponentProps<"button">,
    VariantProps<typeof toggleVariants> {
  /** Controlled pressed state; omitted means managed internally */
  pressed?: boolean;
  /** Initial pressed state in uncontrolled mode */
  defaultPressed?: boolean;
  onPressedChange?: (pressed: boolean) => void;
  /** Unique value used within a ToggleGroup */
  value?: string;
}

import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "solid-js";
import type { toggleVariants } from "~/components/ui/toggle/Toggle/Toggle.styles";
import type { HTMLAttributes, PolymorphicProps } from "~/types";
import type { ToggleGroupValue } from "../ToggleGroup/ToggleGroup.types";

/**
 * The native `value` (declared by Solid as `string | undefined`) must first
 * be omitted from the button props, otherwise it would intersect with
 * `value: TValue` below: `number & string` collapses to `never`.
 */
type BaseToggleGroupItemProps<TValue extends ToggleGroupValue> = VariantProps<
  typeof toggleVariants
> &
  Omit<HTMLAttributes<"button">, "value"> & {
    /** Unique value of this item; its type is set by the generic parameter (matching the owning ToggleGroup) */
    value: TValue;
    /** The component handles selection first, then calls the user callback (same as the tabs trigger) */
    onClick?: ComponentProps<"button">["onClick"];
    /** Focus updates the roving focus highlight; the user callback is invoked as well */
    onFocus?: ComponentProps<"button">["onFocus"];
  };

/**
 * The generic parameter constrains the type of `value` (defaults to
 * `string | number`). A child component cannot infer the type from its parent
 * `ToggleGroup`, so it is inferred from `value` itself here, or specified
 * explicitly: `<ToggleGroupItem<"bold" | "italic"> value="bold" />`.
 */
export type ToggleGroupItemProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> = PolymorphicProps<"button", BaseToggleGroupItemProps<TValue>, false>;

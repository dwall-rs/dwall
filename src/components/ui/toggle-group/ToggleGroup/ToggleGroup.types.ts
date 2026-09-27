import type { VariantProps } from "class-variance-authority";
import type { Accessor } from "solid-js";
import type { toggleVariants } from "~/components/ui/toggle/Toggle/Toggle.styles";
import type { BaseProps, PolymorphicProps } from "~/types";

/**
 * Value type for ToggleGroup / ToggleGroupItem (consistent with `TabsValue`,
 * supporting string and number). Components narrow it to a concrete literal
 * union via the generic parameter, e.g. `"bold" | "italic"` or `1 | 2`.
 */
export type ToggleGroupValue = string | number;

/** Item variant / size values are taken directly from toggleVariants to avoid maintaining them in multiple places */
export type ToggleGroupVariant = NonNullable<
  VariantProps<typeof toggleVariants>["variant"]
>;
export type ToggleGroupSize = NonNullable<
  VariantProps<typeof toggleVariants>["size"]
>;

export type ToggleGroupOrientation = "horizontal" | "vertical";

/** Aligned with base-ui: ToggleGroup's change reason is currently only "none" */
export type ToggleGroupChangeEventReason = "none";

/**
 * Second argument to onValueChange, semantically aligned with base-ui's
 * `ToggleGroup.ChangeEventDetails`.
 *
 * - `cancel()`: prevents the component from committing this change (in
 *   controlled mode state is decided externally, so the component commits
 *   nothing anyway)
 * - `allowPropagation()`: kept for API alignment; this implementation does
 *   not actively stop event propagation, so it is a no-op
 */
export interface ToggleGroupChangeEventDetails {
  reason: ToggleGroupChangeEventReason;
  event: Event;
  cancel: () => void;
  allowPropagation: () => void;
  readonly isCanceled: boolean;
  readonly isPropagationAllowed: boolean;
  trigger: Element | undefined;
}

/** Props shared by single and multiple modes */
interface ToggleGroupCommonProps extends BaseProps {
  /** Disables the whole group; items inherit this state */
  disabled?: boolean;
  /** Layout orientation, defaults to "horizontal"; also decides arrow-key mapping and arrangement direction */
  orientation?: ToggleGroupOrientation;
  /** Whether arrow keys wrap around to the other end at the boundary, defaults to true */
  loopFocus?: boolean;
  /**
   * Gap between items, implemented via the `--gap` variable on the root
   * element, defaults to 2 (aligned with shadcn). Passing 0 joins adjacent
   * items into one block (only the first and last keep rounded corners/borders).
   */
  spacing?: number;
  /** Group-level variant, inherited by items that do not specify their own */
  variant?: ToggleGroupVariant;
  /** Group-level size, inherited by items that do not specify their own */
  size?: ToggleGroupSize;
}

/**
 * Single mode (default): `value` / `defaultValue` / `onValueChange` are all
 * **scalars**.
 *
 * This is an intentional deviation from base-ui — base-ui also requires an
 * array in single mode, which is awkward to use. The callback receives
 * `undefined` when deselecting.
 *
 * Note: `value === undefined` is treated as uncontrolled, so for a controlled
 * single group make sure the signal has a definite value (a signal typed
 * `TValue | undefined` is recommended, backed by `??` before passing, or use
 * the multiple array mode instead).
 */
export interface ToggleGroupSingleProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> extends ToggleGroupCommonProps {
  /** Single-mode marker (default); no need to pass explicitly */
  multiple?: false;
  /** Controlled selected value; omitted means managed internally */
  value?: TValue;
  /** Initial selected value in uncontrolled mode */
  defaultValue?: TValue;
  /** Selected-value change callback (fires in both controlled and uncontrolled modes) */
  onValueChange?: (
    value: TValue | undefined,
    eventDetails: ToggleGroupChangeEventDetails,
  ) => void;
}

/** Multiple mode (`multiple`): the value is an array, empty when nothing is selected */
export interface ToggleGroupMultipleProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> extends ToggleGroupCommonProps {
  /** Multiple-mode marker */
  multiple: true;
  /** Controlled selected value; omitted means managed internally */
  value?: readonly TValue[];
  /** Initial selected value in uncontrolled mode */
  defaultValue?: readonly TValue[];
  /** Selected-value change callback (fires in both controlled and uncontrolled modes) */
  onValueChange?: (
    value: TValue[],
    eventDetails: ToggleGroupChangeEventDetails,
  ) => void;
}

/**
 * The value type is determined by the generic parameter:
 * - omitted, it is the wide type `ToggleGroupValue` (string | number)
 * - a concrete type in any of `defaultValue` / `value` / `onValueChange`
 *   is enough to infer it, e.g. `defaultValue={1}` infers `number`
 * - it can also be specified explicitly: `<ToggleGroup<"bold" | "italic"> />`
 *
 * `multiple` determines the value's shape: a scalar in single mode, an array
 * in multiple mode (passing the wrong one is caught by the types).
 */
export type ToggleGroupProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> = PolymorphicProps<
  "div",
  ToggleGroupSingleProps<TValue> | ToggleGroupMultipleProps<TValue>,
  false
>;

/** Union of the onValueChange signatures in both modes (for the internal state implementation) */
export type ToggleGroupChangeHandler<TValue extends ToggleGroupValue> =
  | NonNullable<ToggleGroupSingleProps<TValue>["onValueChange"]>
  | NonNullable<ToggleGroupMultipleProps<TValue>["onValueChange"]>;

/** A registered item, used for roving focus and keyboard navigation */
export interface ToggleGroupItemEntry<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> {
  value: TValue;
  element: HTMLElement;
  /** accessor, supporting runtime disabled flips */
  disabled: () => boolean;
}

export interface ToggleGroupContextValue<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> {
  /**
   * Current selected values, **already normalized to an array**: 0 or 1
   * elements in single mode. Use `isPressed` to check whether an item is
   * selected.
   */
  value: Accessor<readonly TValue[]>;
  multiple: Accessor<boolean>;
  disabled: Accessor<boolean>;
  orientation: Accessor<ToggleGroupOrientation>;
  loopFocus: Accessor<boolean>;
  dir: Accessor<"ltr" | "rtl" | "auto" | undefined>;
  spacing: Accessor<number>;
  /** Group-level variant / size; undefined when not specified, so items can override them individually */
  variant: Accessor<ToggleGroupVariant | undefined>;
  size: Accessor<ToggleGroupSize | undefined>;
  /** Currently highlighted item (the roving focus tabindex starting point) */
  highlightedValue: Accessor<TValue | undefined>;
  /**
   * The value-taking methods below are declared with method shorthand: method
   * parameters are bivariant, so `ToggleGroupContextValue<number>` remains
   * assignable to `ToggleGroupContextValue<string | number>` (the Provider's
   * value type is the wide type, so function-property syntax cannot be used
   * or contravariance would error).
   */
  /** Whether this value is currently pressed */
  isPressed(value: TValue): boolean;
  /** Toggles an item's pressed state and dispatches onValueChange per mode */
  toggleItem(value: TValue, event: Event, trigger?: Element): void;
  setHighlightedValue(value: TValue): void;
  /** Registers an item in mount order, returning the unregister function */
  registerItem(entry: ToggleGroupItemEntry<TValue>): () => void;
  /** List of registered items (in mount order) */
  getItems(): ToggleGroupItemEntry<TValue>[];
  /** When the highlight is missing, whether this value is the first enabled item (the tabindex starting point) */
  isFirstEnabled(value: TValue): boolean;
  /** Whether the item for this value is currently usable (registered and not disabled) */
  isUsable(value: TValue): boolean;
}

import { createContext, createSignal, useContext } from "solid-js";
import { createChangeEventDetails } from "./create-change-event-details";
import type {
  ToggleGroupChangeHandler,
  ToggleGroupContextValue,
  ToggleGroupItemEntry,
  ToggleGroupMultipleProps,
  ToggleGroupOrientation,
  ToggleGroupSingleProps,
  ToggleGroupSize,
  ToggleGroupValue,
  ToggleGroupVariant,
} from "./ToggleGroup.types";

const ToggleGroupContext = createContext<ToggleGroupContextValue>();

/** onValueChange signatures for single/multiple (derived from the props types to keep a single source of truth) */
type SingleChangeHandler<TValue extends ToggleGroupValue> = NonNullable<
  ToggleGroupSingleProps<TValue>["onValueChange"]
>;
type MultipleChangeHandler<TValue extends ToggleGroupValue> = NonNullable<
  ToggleGroupMultipleProps<TValue>["onValueChange"]
>;

/**
 * Input to createToggleGroupState: the runtime shape of ToggleGroupProps.
 *
 * With a controlled value, single mode receives a scalar and multiple mode an
 * array; internally both are normalized to an array. `multiple` also decides
 * which signature onValueChange is called with.
 */
export interface ToggleGroupStateProps<
  TValue extends ToggleGroupValue = ToggleGroupValue,
> {
  value?: TValue | readonly TValue[];
  defaultValue?: TValue | readonly TValue[];
  onValueChange?: ToggleGroupChangeHandler<TValue>;
  multiple?: boolean;
  disabled?: boolean;
  orientation?: ToggleGroupOrientation;
  loopFocus?: boolean;
  dir?: "ltr" | "rtl" | "auto";
  spacing?: number;
  variant?: ToggleGroupVariant;
  size?: ToggleGroupSize;
}

/**
 * The single source of state for ToggleGroup: selected value
 * (controlled/uncontrolled), focus highlight, and the item registry.
 *
 * Every field on the returned context value is an accessor, so the `props`
 * passed in can be the reactive proxy from splitProps and reads always see
 * the latest value. The component root only emits DOM and data-* style hooks;
 * state and interaction logic converge here.
 *
 * The value type is determined by the generic parameter; internally only
 * `===` / `includes` are used for identity comparison, so both string and
 * number work.
 */
export function createToggleGroupState<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(props: ToggleGroupStateProps<TValue>): ToggleGroupContextValue<TValue> {
  const multiple = () => props.multiple === true;

  /** Single passes a scalar, multiple an array; normalized to an array internally */
  const toArray = (v: TValue | readonly TValue[] | undefined): TValue[] => {
    if (v === undefined) return [];
    return Array.isArray(v) ? [...(v as readonly TValue[])] : [v as TValue];
  };

  const [internalValue, setInternalValue] = createSignal<TValue[]>(
    toArray(props.defaultValue),
  );
  const [highlightedValue, setHighlightedValue] = createSignal<
    TValue | undefined
  >();
  const [itemOrder, setItemOrder] = createSignal<
    ToggleGroupItemEntry<TValue>[]
  >([]);

  const isControlled = () => props.value !== undefined;
  // Controlled takes priority: read the external value when props.value is present, otherwise internal state
  const value = (): readonly TValue[] =>
    isControlled() ? toArray(props.value) : internalValue();

  const isPressed = (v: TValue) => value().includes(v);

  const toggleItem = (
    itemValue: TValue,
    event: Event,
    trigger?: Element,
  ): void => {
    const current = value();
    const pressed = current.includes(itemValue);

    // In single mode, clicking an already pressed item deselects it (next = [])
    let next: TValue[];
    if (multiple()) {
      next = pressed
        ? current.filter((v) => v !== itemValue)
        : [...current, itemValue];
    } else {
      next = pressed ? [] : [itemValue];
    }

    // Notify the outside first: cancel() inside onValueChange can prevent the
    // component from committing; single gets a scalar (undefined on cancel), multiple an array
    const details = createChangeEventDetails(event, trigger);
    if (multiple()) {
      (props.onValueChange as MultipleChangeHandler<TValue> | undefined)?.(
        next,
        details,
      );
    } else {
      (props.onValueChange as SingleChangeHandler<TValue> | undefined)?.(
        next[0],
        details,
      );
    }
    if (details.isCanceled) return;

    // In controlled mode only the outside is notified; internal state is not changed
    if (!isControlled()) setInternalValue(next);
  };

  const registerItem = (entry: ToggleGroupItemEntry<TValue>) => {
    setItemOrder((prev) => [...prev, entry]);
    return () => setItemOrder((prev) => prev.filter((item) => item !== entry));
  };

  const isFirstEnabled = (v: TValue) =>
    itemOrder().find((item) => !item.disabled())?.value === v;

  const isUsable = (v: TValue) =>
    itemOrder().some((item) => item.value === v && !item.disabled());

  return {
    value,
    multiple,
    disabled: () => props.disabled === true,
    orientation: () => props.orientation ?? "horizontal",
    loopFocus: () => props.loopFocus ?? true,
    dir: () => props.dir,
    spacing: () => props.spacing ?? 2,
    variant: () => props.variant,
    size: () => props.size,
    highlightedValue,
    isPressed,
    toggleItem,
    setHighlightedValue,
    registerItem,
    getItems: () => itemOrder(),
    isFirstEnabled,
    isUsable,
  };
}

/**
 * Reads the ToggleGroup context.
 *
 * The generic parameter is **annotated by the caller**: the component cannot
 * derive the group's value type from its children, so when writing a custom
 * item pass a type matching ToggleGroup explicitly, e.g.
 * `useToggleGroupContext<"bold" | "italic">("MyToggle")`.
 */
export function useToggleGroupContext<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(component: string): ToggleGroupContextValue<TValue> {
  const ctx = useContext(ToggleGroupContext);
  if (!ctx) {
    throw new Error(`<${component}> must be rendered inside <ToggleGroup>`);
  }
  return ctx as ToggleGroupContextValue<TValue>;
}

export { ToggleGroupContext };

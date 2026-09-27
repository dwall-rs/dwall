import { createSignal, createMemo, createUniqueId, type JSX } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { useScrollLock } from "~/hooks/useScrollLock";
import { SelectContext } from "./Select.context";
import type {
  SelectContextValue,
  SelectItemMeta,
  SelectOptionValue,
  SelectProps,
} from "./Select.types";

/**
 * Select root component: renders no DOM; it only manages state and
 * provides context. Actual rendering is delegated to
 * <SelectTrigger>/<SelectValue>/<SelectContent>/<SelectItem>.
 *
 * The generic T is constrained to string | number, bounding the types of
 * value / defaultValue / onValueChange / SelectItem.value.
 *
 * @example
 * ```tsx
 * <Select value={fruit()} onValueChange={setFruit}>
 *   <SelectTrigger>
 *     <SelectValue placeholder="Pick a fruit" />
 *   </SelectTrigger>
 *   <SelectContent>
 *     <SelectItem value="apple">Apple</SelectItem>
 *   </SelectContent>
 * </Select>
 * ```
 */
export function Select<T extends SelectOptionValue = string>(
  props: SelectProps<T>,
): JSX.Element {
  const [internalValue, setInternalValue] = createSignal<T | undefined>(
    props.defaultValue,
  );
  const value = createMemo(() =>
    props.value !== undefined ? props.value : internalValue(),
  );

  const setValue = (v: T) => {
    if (props.value === undefined) setInternalValue(() => v);
    props.onValueChange?.(v);
  };

  const [internalOpen, setInternalOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const open = createMemo(() =>
    props.open !== undefined ? props.open : internalOpen(),
  );
  const disabled = createMemo(() => !!props.disabled);
  const lockScroll = createMemo(() => props.lockScroll ?? true);

  // While open, page scrolling is locked by default: document scrolling is
  // locked and wheel/touch events outside the overlay are intercepted, while
  // the panel itself remains scrollable. Pass lockScroll={false} to disable.
  useScrollLock(() => open() && lockScroll(), {
    allowedSelector: '[data-slot="select-content"]',
  });

  const setOpen = (next: boolean) => {
    if (props.open === undefined) setInternalOpen(next);
    props.onOpenChange?.(next);
  };

  const [reference, setReference] = createSignal<Element>();
  const [floating, setFloating] = createSignal<HTMLElement>();
  const [activeValue, setActiveValue] = createSignal<T>();
  const [items, setItems] = createStore<SelectItemMeta<SelectOptionValue>[]>(
    [],
  );
  const contentId = `select-content-${createUniqueId()}`;

  const registerItem = (item: SelectItemMeta<SelectOptionValue>) => {
    setItems(produce((list) => list.push(item)));
    return () =>
      setItems(
        produce((list) => {
          const idx = list.findIndex((it) => it.value === item.value);
          if (idx !== -1) list.splice(idx, 1);
        }),
      );
  };

  const close = () => setOpen(false);

  const closeAndFocusTrigger = () => {
    setOpen(false);
    (reference() as HTMLElement | undefined)?.focus?.();
  };

  const selectValue = (v: SelectOptionValue) => {
    setValue(v as T);
    closeAndFocusTrigger();
  };

  const ctx: SelectContextValue = {
    value: value as SelectContextValue["value"],
    setValue: setValue as SelectContextValue["setValue"],
    open,
    setOpen,
    disabled,
    contentId,
    reference,
    setReference,
    floating,
    setFloating,
    get items() {
      return items;
    },
    registerItem,
    activeValue: activeValue as SelectContextValue["activeValue"],
    setActiveValue: setActiveValue as SelectContextValue["setActiveValue"],
    selectValue,
    close,
  };

  return (
    <SelectContext.Provider value={ctx}>
      {props.children}
    </SelectContext.Provider>
  );
}

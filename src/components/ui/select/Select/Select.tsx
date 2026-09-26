import { createSignal, createMemo, createUniqueId, type JSX } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { SelectContext } from "./Select.context";
import type {
  SelectContextValue,
  SelectItemMeta,
  SelectOptionValue,
  SelectProps,
} from "./Select.types";

/**
 * Select 根组件：不渲染任何 DOM，只负责状态管理 + 提供 context。
 * 真正的展示交给 <SelectTrigger>/<SelectValue>/<SelectContent>/<SelectItem>。
 *
 * 泛型 T 约束为 string | number，用于约束 value / defaultValue /
 * onValueChange / SelectItem.value 的类型。
 *
 * @example
 * ```tsx
 * <Select value={fruit()} onValueChange={setFruit}>
 *   <SelectTrigger>
 *     <SelectValue placeholder="选择一个水果" />
 *   </SelectTrigger>
 *   <SelectContent>
 *     <SelectItem value="apple">苹果</SelectItem>
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

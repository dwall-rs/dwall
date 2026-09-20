import { createSignal, createMemo, createUniqueId, type JSX } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { SelectContext } from "./Select.context";
import type {
  SelectContextValue,
  SelectItemMeta,
  SelectProps,
} from "./Select.types";

/**
 * Select 根组件：不渲染任何 DOM，只负责状态管理 + 提供 context。
 * 真正的展示交给 <SelectTrigger>/<SelectValue>/<SelectContent>/<SelectItem>。
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
export function Select(props: SelectProps): JSX.Element {
  const [internalValue, setInternalValue] = createSignal(props.defaultValue);
  const value = createMemo(() =>
    props.value !== undefined ? props.value : internalValue(),
  );

  const setValue = (v: string) => {
    if (props.value === undefined) setInternalValue(v);
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
  const [activeValue, setActiveValue] = createSignal<string>();
  const [items, setItems] = createStore<SelectItemMeta[]>([]);
  const contentId = `select-content-${createUniqueId()}`;

  const registerItem = (item: SelectItemMeta) => {
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

  const selectValue = (v: string) => {
    setValue(v);
    closeAndFocusTrigger();
  };

  const ctx: SelectContextValue = {
    value,
    setValue,
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
    activeValue,
    setActiveValue,
    selectValue,
    close,
  };

  return (
    <SelectContext.Provider value={ctx}>
      {props.children}
    </SelectContext.Provider>
  );
}

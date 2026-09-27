import type { Accessor, ParentProps } from "solid-js";

export type SelectOptionValue = string | number;

export interface SelectItemMeta<
  T extends SelectOptionValue = SelectOptionValue,
> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SelectProps<T extends SelectOptionValue = string>
  extends ParentProps {
  /**
   * Controlled selected value; pass null to clear the selection while in
   * controlled mode. Omit (undefined) to let the component manage the value
   * internally (uncontrolled mode).
   */
  value?: T | null;
  /** Initial selected value in uncontrolled mode */
  defaultValue?: T;
  onValueChange?: (value: T) => void;
  /** Controlled open state; omitted means managed internally */
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
  /**
   * Whether to lock page scrolling while open, defaults to true: document
   * scrolling is locked and wheel/touch scrolling outside the overlay is
   * intercepted, while the panel itself remains scrollable.
   */
  lockScroll?: boolean;
}

export interface SelectContextValue<
  T extends SelectOptionValue = SelectOptionValue,
> {
  value: Accessor<T | null | undefined>;
  setValue: (v: T) => void;
  open: Accessor<boolean>;
  setOpen: (v: boolean) => void;
  disabled: Accessor<boolean>;
  contentId: string;
  reference: Accessor<Element | undefined>;
  setReference: (el: Element | undefined) => void;
  floating: Accessor<HTMLElement | undefined>;
  setFloating: (el: HTMLElement | undefined) => void;
  /** Registered option list (reactive store), read by SelectContent/SelectValue */
  items: SelectItemMeta<T>[];
  /** SelectItem registers itself on mount and returns an unregister function (called on unmount) */
  registerItem: (item: SelectItemMeta<T>) => () => void;
  /** Item highlighted by keyboard/hover */
  activeValue: Accessor<T | undefined>;
  setActiveValue: (v: T | undefined) => void;
  /** Called after an item is chosen: set the value, close, return focus to the trigger */
  selectValue: (v: T) => void;
  /** Plain close (selects no value), used for Esc / clicking outside */
  close: () => void;
}

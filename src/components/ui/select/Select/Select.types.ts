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
  value?: T | null;

  defaultValue?: T;
  onValueChange?: (value: T) => void;

  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;

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

  items: SelectItemMeta<T>[];

  registerItem: (item: SelectItemMeta<T>) => () => void;

  activeValue: Accessor<T | undefined>;
  setActiveValue: (v: T | undefined) => void;

  selectValue: (v: T) => void;

  close: () => void;
}

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
   * 受控选中值；传入 null 表示受控模式下清空已选值。
   * 不传（undefined）则内部自管理（非受控模式）。
   */
  value?: T | null;
  /** 非受控模式下的初始选中值 */
  defaultValue?: T;
  onValueChange?: (value: T) => void;
  /** 受控 open 状态；不传则内部自管理 */
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
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
  /** 已注册的选项列表（响应式 store），SelectContent/SelectValue 都要读 */
  items: SelectItemMeta<T>[];
  /** SelectItem 挂载时注册自己，返回取消注册函数（卸载时调用） */
  registerItem: (item: SelectItemMeta<T>) => () => void;
  /** 键盘/鼠标悬停高亮的项 */
  activeValue: Accessor<T | undefined>;
  setActiveValue: (v: T | undefined) => void;
  /** 选中某项后调用：设值、关闭、把 focus 还给 trigger */
  selectValue: (v: T) => void;
  /** 单纯关闭（不选择任何值），键盘 Esc / 点击外部时用 */
  close: () => void;
}

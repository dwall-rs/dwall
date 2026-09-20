import type { Accessor, ParentProps } from "solid-js";

export interface SelectItemMeta {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends ParentProps {
  /** 受控选中值；不传则内部自管理（非受控模式） */
  value?: string;
  /** 非受控模式下的初始选中值 */
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  /** 受控 open 状态；不传则内部自管理 */
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
}

export interface SelectContextValue {
  value: Accessor<string | undefined>;
  setValue: (v: string) => void;
  open: Accessor<boolean>;
  setOpen: (v: boolean) => void;
  disabled: Accessor<boolean>;
  contentId: string;
  reference: Accessor<Element | undefined>;
  setReference: (el: Element | undefined) => void;
  floating: Accessor<HTMLElement | undefined>;
  setFloating: (el: HTMLElement | undefined) => void;
  /** 已注册的选项列表（响应式 store），SelectContent/SelectValue 都要读 */
  items: SelectItemMeta[];
  /** SelectItem 挂载时注册自己，返回取消注册函数（卸载时调用） */
  registerItem: (item: SelectItemMeta) => () => void;
  /** 键盘/鼠标悬停高亮的项 */
  activeValue: Accessor<string | undefined>;
  setActiveValue: (v: string | undefined) => void;
  /** 选中某项后调用：设值、关闭、把 focus 还给 trigger */
  selectValue: (v: string) => void;
  /** 单纯关闭（不选择任何值），键盘 Esc / 点击外部时用 */
  close: () => void;
}

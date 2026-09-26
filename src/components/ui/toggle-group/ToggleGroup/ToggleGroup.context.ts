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

/** 单选/多选的 onValueChange 签名(从 props 类型派生,保持单一事实来源) */
type SingleChangeHandler<TValue extends ToggleGroupValue> = NonNullable<
  ToggleGroupSingleProps<TValue>["onValueChange"]
>;
type MultipleChangeHandler<TValue extends ToggleGroupValue> = NonNullable<
  ToggleGroupMultipleProps<TValue>["onValueChange"]
>;

/**
 * createToggleGroupState 的输入:ToggleGroupProps 的运行时形态。
 *
 * 受控值时单选给标量、多选给数组,内部统一规范化成数组;
 * `multiple` 同时决定 onValueChange 用哪种签名回调。
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
 * ToggleGroup 的唯一状态源:选中值(受控/非受控)、焦点高亮、item 注册表。
 *
 * 返回的 context 值里所有字段都是 accessor,因此传入的 `props` 可以是
 * splitProps 得到的响应式代理,读取始终是最新值。
 * 组件根只负责输出 DOM 与 data-* 样式钩子,状态与交互算法收敛在这里。
 *
 * 值类型由泛型参数决定;内部只用 `===` / `includes` 做同一性比较,
 * 因此 string 与 number 都能正常工作。
 */
export function createToggleGroupState<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(props: ToggleGroupStateProps<TValue>): ToggleGroupContextValue<TValue> {
  const multiple = () => props.multiple === true;

  /** 单选传标量、多选传数组,内部统一成数组 */
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
  // 受控优先:props.value 存在时读外部值,否则读内部状态
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

    // 单选时再次点击已按下项会取消选中(next = [])
    let next: TValue[];
    if (multiple()) {
      next = pressed
        ? current.filter((v) => v !== itemValue)
        : [...current, itemValue];
    } else {
      next = pressed ? [] : [itemValue];
    }

    // 先通知外部:onValueChange 中 cancel() 可阻止组件提交本次变更
    // 单选回调标量(取消时是 undefined),多选回调数组
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

    // 受控模式只通知外部,不改内部状态
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
 * 读取 ToggleGroup 的 context。
 *
 * 泛型参数是**调用方标注**:组件本身无法从子节点推导所属 group 的值类型,
 * 自定义 item 时请显式传入与 ToggleGroup 一致的类型,例如
 * `useToggleGroupContext<"bold" | "italic">("MyToggle")`。
 */
export function useToggleGroupContext<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(component: string): ToggleGroupContextValue<TValue> {
  const ctx = useContext(ToggleGroupContext);
  if (!ctx) {
    throw new Error(`<${component}> 必须渲染在 <ToggleGroup> 内部`);
  }
  return ctx as ToggleGroupContextValue<TValue>;
}

export { ToggleGroupContext };

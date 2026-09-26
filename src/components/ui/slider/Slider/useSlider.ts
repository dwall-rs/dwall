import { createMemo, createSignal } from "solid-js";
import type { SliderProps } from "./Slider.types";
import type { NonNullableProps } from "~/types";

export const useSlider = <T extends number | number[]>({
  local,
}: {
  local: NonNullableProps<
    Pick<
      SliderProps<T>,
      | "defaultValue"
      | "value"
      | "max"
      | "min"
      | "step"
      | "orientation"
      | "onValueChange"
      | "disabled"
    >,
    "max" | "min" | "step" | "orientation" | "disabled"
  >;
}) => {
  // 由初始传入的类型一次性决定，运行时不再改变
  const isArray = Array.isArray(local.value ?? local.defaultValue);

  // 将外部的 T 统一转为内部 number[]
  const toArray = (v: T | undefined): number[] => {
    if (v === undefined) return [local.min];
    return Array.isArray(v) ? v : [v as number];
  };

  // 受控 / 非受控模式
  const isControlled = () => local.value !== undefined;

  const [internalValues, setInternalValues] = createSignal<number[]>(
    toArray(local.defaultValue),
  );

  const values = createMemo<number[]>(() =>
    isControlled() ? toArray(local.value) : internalValues(),
  );

  // 将内部 number[] 还原为外部类型 T
  const toExternal = (arr: number[]): T => (isArray ? arr : arr[0]) as T;

  // 当前被拖拽的 thumb 索引
  const [activeThumbIndex, setActiveThumbIndex] = createSignal(0);

  // track DOM 引用（由 SliderTrack 注入）
  let trackEl: HTMLDivElement | undefined;
  const trackRef = () => trackEl;
  const setTrackRef = (el: HTMLDivElement) => {
    trackEl = el;
  };

  /** 从 step 推导需要保留的小数位数，例如 step=0.05 → 2 位 */
  const stepDecimals = createMemo(() => {
    const s = local.step.toString();
    const dot = s.indexOf(".");
    return dot === -1 ? 0 : s.length - dot - 1;
  });

  /** 将原始数值 snap 到 step 并 clamp 到 [min, max] */
  const snapValue = (raw: number) => {
    const { min, max, step } = local;
    const snapped = Math.round((raw - min) / step) * step + min;
    const fixed = parseFloat(snapped.toFixed(stepDecimals()));
    return Math.min(max, Math.max(min, fixed));
  };

  const updateValue = (index: number, rawValue: number) => {
    const next = values().slice();
    let snapped = snapValue(rawValue);

    // 防止 thumbs 交叉（多 thumb 场景）
    if (index > 0 && snapped <= next[index - 1]) {
      snapped = snapValue(next[index - 1] + local.step);
    }
    if (index < next.length - 1 && snapped >= next[index + 1]) {
      snapped = snapValue(next[index + 1] - local.step);
    }

    next[index] = snapped;

    if (!isControlled()) setInternalValues(next);
    local.onValueChange?.(toExternal(next));
  };

  const context = {
    values,
    min: () => local.min,
    max: () => local.max,
    step: () => local.step,
    orientation: () => local.orientation,
    disabled: () => local.disabled,
    activeThumbIndex,
    setActiveThumbIndex,
    updateValue,
    trackRef,
    setTrackRef,
  };

  return { context };
};

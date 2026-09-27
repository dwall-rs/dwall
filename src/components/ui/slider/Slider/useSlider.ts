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
  // Decided once from the initially passed type; never changes at runtime
  const isArray = Array.isArray(local.value ?? local.defaultValue);

  // Normalize the external T into an internal number[]
  const toArray = (v: T | undefined): number[] => {
    if (v === undefined) return [local.min];
    return Array.isArray(v) ? v : [v as number];
  };

  // Controlled / uncontrolled mode
  const isControlled = () => local.value !== undefined;

  const [internalValues, setInternalValues] = createSignal<number[]>(
    toArray(local.defaultValue),
  );

  const values = createMemo<number[]>(() =>
    isControlled() ? toArray(local.value) : internalValues(),
  );

  // Restore the internal number[] back to the external type T
  const toExternal = (arr: number[]): T => (isArray ? arr : arr[0]) as T;

  // Index of the thumb currently being dragged
  const [activeThumbIndex, setActiveThumbIndex] = createSignal(0);

  // Track DOM reference (injected by SliderTrack)
  let trackEl: HTMLDivElement | undefined;
  const trackRef = () => trackEl;
  const setTrackRef = (el: HTMLDivElement) => {
    trackEl = el;
  };

  /** Derive the number of decimal places to keep from step, e.g. step=0.05 → 2 */
  const stepDecimals = createMemo(() => {
    const s = local.step.toString();
    const dot = s.indexOf(".");
    return dot === -1 ? 0 : s.length - dot - 1;
  });

  /** Snap the raw value to step and clamp it to [min, max] */
  const snapValue = (raw: number) => {
    const { min, max, step } = local;
    const snapped = Math.round((raw - min) / step) * step + min;
    const fixed = parseFloat(snapped.toFixed(stepDecimals()));
    return Math.min(max, Math.max(min, fixed));
  };

  const updateValue = (index: number, rawValue: number) => {
    const next = values().slice();
    let snapped = snapValue(rawValue);

    // Prevent thumbs from crossing (multi-thumb case)
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

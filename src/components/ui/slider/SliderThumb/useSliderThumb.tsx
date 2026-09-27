import { createMemo } from "solid-js";
import { useSliderContext } from "../Slider";

export const useSliderThumb = (index: number) => {
  const ctx = useSliderContext();

  const value = createMemo(() => ctx.values()[index] ?? ctx.min());

  /**
   * The thumb is absolutely positioned relative to SliderControl
   * (position: relative).
   * Main axis: percentage + translate(-50%) centers the thumb on the tick point.
   * Cross axis: 50% + translate(-50%) keeps the thumb centered on the track line.
   */
  const positionStyle = createMemo(() => {
    const pct = ((value() - ctx.min()) / (ctx.max() - ctx.min())) * 100;

    return {
      "--position": `${pct}%`,
    };
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (ctx.disabled()) return;

    const step = e.shiftKey ? ctx.step() * 10 : ctx.step();
    const isVertical = ctx.orientation() === "vertical";

    const increaseKeys = isVertical ? ["ArrowUp"] : ["ArrowRight", "ArrowUp"];
    const decreaseKeys = isVertical
      ? ["ArrowDown"]
      : ["ArrowLeft", "ArrowDown"];

    if (increaseKeys.includes(e.key)) {
      e.preventDefault();
      ctx.updateValue(index, value() + step);
    } else if (decreaseKeys.includes(e.key)) {
      e.preventDefault();
      ctx.updateValue(index, value() - step);
    } else if (e.key === "Home") {
      e.preventDefault();
      ctx.updateValue(index, ctx.min());
    } else if (e.key === "End") {
      e.preventDefault();
      ctx.updateValue(index, ctx.max());
    }
  };

  return { positionStyle, handleKeyDown };
};

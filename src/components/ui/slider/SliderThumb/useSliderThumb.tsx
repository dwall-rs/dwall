import { createMemo } from "solid-js";
import { useSliderContext } from "../Slider";

export const useSliderThumb = (index: number) => {
  const ctx = useSliderContext();

  const value = createMemo(() => ctx.values()[index] ?? ctx.min());

  /**
   * thumb 相对于 SliderControl（position: relative）绝对定位。
   * 主轴：百分比 + translate(-50%) 使 thumb 中心对齐刻度点。
   * 交叉轴：50% + translate(-50%) 使 thumb 始终居中于 track 细线。
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

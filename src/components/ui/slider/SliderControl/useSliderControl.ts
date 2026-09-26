import { useSliderContext } from "../Slider";

export const useSliderControl = () => {
  const ctx = useSliderContext();

  /** 将页面坐标转为 slider 值 */
  const coordToValue = (clientX: number, clientY: number): number => {
    const track = ctx.trackRef();
    if (!track) return ctx.min();

    const rect = track.getBoundingClientRect();
    const isVertical = ctx.orientation() === "vertical";

    let ratio: number;
    if (isVertical) {
      // 垂直方向：底部为 min，顶部为 max
      ratio = 1 - (clientY - rect.top) / rect.height;
    } else {
      ratio = (clientX - rect.left) / rect.width;
    }

    return ctx.min() + ratio * (ctx.max() - ctx.min());
  };

  /** 找到距离点击位置最近的 thumb 索引 */
  const closestThumbIndex = (value: number): number => {
    const vals = ctx.values();
    let closest = 0;
    let minDist = Infinity;
    vals.forEach((v, i) => {
      const dist = Math.abs(v - value);
      if (dist < minDist) {
        minDist = dist;
        closest = i;
      }
    });
    return closest;
  };

  const handlePointerDown = (e: PointerEvent) => {
    if (ctx.disabled()) return;
    if (e.button !== 0) return;

    const value = coordToValue(e.clientX, e.clientY);
    const index = closestThumbIndex(value);

    ctx.setActiveThumbIndex(index);
    ctx.updateValue(index, value);

    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
  };

  const handlePointerMove = (e: PointerEvent) => {
    if (ctx.disabled()) return;
    // 只有在 pointer 被捕获（即 pointerdown 后）才处理
    if (!(e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId))
      return;

    const value = coordToValue(e.clientX, e.clientY);
    ctx.updateValue(ctx.activeThumbIndex(), value);
  };

  const handlePointerUp = (e: PointerEvent) => {
    const el = e.currentTarget as HTMLElement;
    if (el.hasPointerCapture(e.pointerId)) {
      el.releasePointerCapture(e.pointerId);
    }
  };

  return {
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
  };
};

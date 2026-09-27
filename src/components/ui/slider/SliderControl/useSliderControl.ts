import { useSliderContext } from "../Slider";

export const useSliderControl = () => {
  const ctx = useSliderContext();

  /** Convert page coordinates to a slider value */
  const coordToValue = (clientX: number, clientY: number): number => {
    const track = ctx.trackRef();
    if (!track) return ctx.min();

    const rect = track.getBoundingClientRect();
    const isVertical = ctx.orientation() === "vertical";

    let ratio: number;
    if (isVertical) {
      // Vertical: bottom is min, top is max
      ratio = 1 - (clientY - rect.top) / rect.height;
    } else {
      ratio = (clientX - rect.left) / rect.width;
    }

    return ctx.min() + ratio * (ctx.max() - ctx.min());
  };

  /** Find the index of the thumb closest to the click position */
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
    // Only handle once the pointer is captured (i.e. after pointerdown)
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

import type { VirtualElement } from "./types";

export interface PointOptions {
  x: number;
  y: number;
  /** Optional: associate a real DOM node for autoUpdate to find scrollable ancestors */
  contextElement?: Element;
}

/**
 * Create a virtual reference element anchored to specific pixel coordinates
 * (width and height both 0). Typical uses: context menus anchored to the mouse
 * click position, drag-follows-cursor, coordinate-popped tooltips, etc.
 *
 * @example
 * ```ts
 * const [point, setPoint] = createSignal<VirtualElement>();
 * el.addEventListener('contextmenu', (e) => {
 *   e.preventDefault();
 *   setPoint(createVirtualElement({ x: e.clientX, y: e.clientY, contextElement: el }));
 * });
 * ```
 */
export function createVirtualElement(point: PointOptions): VirtualElement {
  return {
    contextElement: point.contextElement,
    getBoundingClientRect() {
      return { x: point.x, y: point.y, width: 0, height: 0 };
    },
  };
}

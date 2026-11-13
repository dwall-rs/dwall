import type { VirtualElement } from "./types";

export interface PointOptions {
  x: number;
  y: number;
  /** 可选：关联一个真实 DOM 节点，供 autoUpdate 查找可滚动祖先使用 */
  contextElement?: Element;
}

/**
 * 创建一个锚定到具体像素坐标的虚拟参照元素（宽高均为 0），
 * 常见用途：右键菜单锚定到鼠标点击位置、拖拽跟随光标、按坐标弹出的提示等。
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

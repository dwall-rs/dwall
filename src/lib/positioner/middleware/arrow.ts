import type { Middleware } from "../types";
import { getSide, isVerticalSide } from "../core/placement";

export interface ArrowOptions {
  /**
   * 箭头 DOM 元素，或返回该元素的函数（Solid 场景下常传一个访问器）。
   * 类型是 Element 而不是 HTMLElement，是为了兼容 SVG 场景下用 <svg> 画箭头的写法
   * （SVGSVGElement 不是 HTMLElement 的子类型）。
   */
  element: Element | undefined | (() => Element | undefined);
  /** 箭头距 floating 边缘的最小间距，默认 4 */
  padding?: number;
}

export interface ArrowData {
  x?: number;
  y?: number;
  /** 箭头应贴靠 floating 的哪一侧（与 floating 的 placement 主边相同） */
  side: ReturnType<typeof getSide>;
}

/**
 * 计算箭头相对 floating 元素左上角的 x/y 偏移，
 * 使其始终指向 reference 元素中心，同时不超出 floating 边界。
 */
export function arrow(options: ArrowOptions): Middleware {
  return {
    name: "arrow",
    fn(state) {
      const el =
        typeof options.element === "function"
          ? options.element()
          : options.element;
      const side = getSide(state.placement);
      if (!el) return { data: { side } };

      const { rects, x, y } = state;
      const padding = options.padding ?? 4;
      const vertical = isVerticalSide(side);

      // 用 getBoundingClientRect 而不是 offsetWidth/offsetHeight：后者只有
      // HTMLElement 才有，SVG 元素（比如 <svg>/<g>）没有这两个属性；
      // getBoundingClientRect 对所有 Element 都通用，只要箭头本身没有做
      // CSS transform（本库的 TooltipArrow 就是特意不用 rotate 来避免这个问题）。
      const arrowRect = el.getBoundingClientRect();
      const arrowWidth = arrowRect.width || 0;
      const arrowHeight = arrowRect.height || 0;

      // 关键：floating 的宽高现读现用，不用 state.rects.floating（那是这一轮
      // computePosition 刚开始时缓存的值）。如果 arrow 前面排了 size 这类会
      // 通过 apply 回调产生 DOM 副作用（比如响应式地改 max-width）的 middleware，
      // 而且这次副作用刚好同步生效了，state.rects.floating 就会是“改之前”的
      // 旧尺寸，用它算出来的箭头位置会带着一个固定的偏移量，跟最终实际渲染的
      // 盒子宽度对不上。这里现场重新measure，保证永远拿到当下最新的尺寸。
      const freshFloatingRect = state.elements.floating.getBoundingClientRect();
      const axisLen = vertical
        ? freshFloatingRect.width
        : freshFloatingRect.height;
      const arrowLen = vertical ? arrowWidth : arrowHeight;

      // reference 中心点相对 floating 左上角的位置
      const refCenter = vertical
        ? rects.reference.x + rects.reference.width / 2 - x
        : rects.reference.y + rects.reference.height / 2 - y;

      let pos = refCenter - arrowLen / 2;
      const min = padding;
      const max = axisLen - arrowLen - padding;
      pos = Math.min(Math.max(pos, min), Math.max(min, max));

      const data: ArrowData = vertical ? { x: pos, side } : { y: pos, side };
      return { data };
    },
  };
}

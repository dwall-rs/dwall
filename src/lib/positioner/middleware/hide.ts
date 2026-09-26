import type { Boundary, Middleware } from "../types";
import { getViewportBoundary, getRectRelativeTo } from "../utils/dom";

export interface HideOptions {
  padding?: number;
  boundary?: Boundary;
  /**
   * 'referenceHidden'：reference 元素被自身滚动容器完全裁掉（比如滚出了可滚动的父级）
   * 'escaped'：floating 元素虽然还在视口内，但已经完全脱离了 reference 所在的边界
   *（比如 reference 在一个卡片里被横向滚动挡住了一半，虽然 floating 还能摆在视口里，
   *  但已经不该再显示，否则看起来像是"飘"在别的内容上）
   * 默认只做 referenceHidden 检测，这是最常见也最需要的场景。
   */
  strategy?: "referenceHidden" | "escaped";
}

export interface HideData {
  referenceHidden?: boolean;
  escaped?: boolean;
}

/**
 * 不改变坐标，只产出"是否应该隐藏浮层"的判断数据，交给上层组件决定要不要渲染。
 * 用法：在业务组件里读 `middlewareData().hide?.referenceHidden`，为 true 时关闭浮层。
 */
export function hide(options: HideOptions = {}): Middleware {
  return {
    name: "hide",
    fn(state) {
      const { x, y, rects, strategy: positionStrategy, elements } = state;
      const { padding = 0, strategy = "referenceHidden" } = options;
      const boundary =
        options.boundary ?? getViewportBoundary(positionStrategy, padding);

      const data: HideData = {};

      if (strategy === "referenceHidden") {
        const refRect = getRectRelativeTo(elements.reference, positionStrategy);
        const referenceHidden =
          refRect.x + refRect.width <= boundary.x ||
          refRect.y + refRect.height <= boundary.y ||
          refRect.x >= boundary.x + boundary.width ||
          refRect.y >= boundary.y + boundary.height;
        data.referenceHidden = referenceHidden;
      }

      if (strategy === "escaped") {
        const escaped =
          x + rects.floating.width <= boundary.x ||
          y + rects.floating.height <= boundary.y ||
          x >= boundary.x + boundary.width ||
          y >= boundary.y + boundary.height;
        data.escaped = escaped;
      }

      return { data };
    },
  };
}

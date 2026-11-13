import type {
  ComputePositionConfig,
  ComputePositionReturn,
  ElementRects,
  Elements,
  Placement,
  ReferenceElement,
  Strategy,
} from "../types";
import { getRectRelativeTo } from "../utils/dom";
import { computeCoordsFromPlacement } from "./placement";

function getElementRects(elements: Elements, strategy: Strategy): ElementRects {
  return {
    reference: getRectRelativeTo(elements.reference, strategy),
    floating: getRectRelativeTo(elements.floating, strategy),
  };
}

const MAX_MIDDLEWARE_PASSES = 8;

/**
 * 计算 floating 元素相对 reference 元素的最终坐标。
 * reference 既可以是真实 DOM 元素，也可以是虚拟参照元素（见 createVirtualElement），
 * 后者常用于锚定到一个坐标点，比如右键菜单的鼠标点击位置。
 * middleware 按顺序执行，任意 middleware 返回 reset 时会用新的 placement 重跑一轮，
 * 但最多重跑 MAX_MIDDLEWARE_PASSES 次，避免死循环。
 */
export function computePosition(
  reference: ReferenceElement,
  floating: HTMLElement,
  config: ComputePositionConfig = {},
): ComputePositionReturn {
  const {
    placement = "bottom",
    strategy = "absolute",
    middleware = [],
  } = config;
  const elements: Elements = { reference, floating };

  let statePlacement: Placement = placement;
  let rects = getElementRects(elements, strategy);
  let { x, y } = computeCoordsFromPlacement(rects, statePlacement);
  let middlewareData: Record<string, any> = {};

  let passes = 0;
  let i = 0;
  while (i < middleware.length) {
    const { name, fn } = middleware[i];
    const result = fn({
      x,
      y,
      initialPlacement: placement,
      placement: statePlacement,
      strategy,
      rects,
      elements,
      middlewareData,
    });

    if (result.x !== undefined) x = result.x;
    if (result.y !== undefined) y = result.y;
    if (result.data !== undefined) {
      middlewareData = { ...middlewareData, [name]: result.data };
    }

    if (result.reset && passes < MAX_MIDDLEWARE_PASSES) {
      passes++;
      if (typeof result.reset === "object" && result.reset.placement) {
        statePlacement = result.reset.placement;
      }
      rects = getElementRects(elements, strategy);
      ({ x, y } = computeCoordsFromPlacement(rects, statePlacement));
      i = 0;
      continue;
    }

    i++;
  }

  return { x, y, placement: statePlacement, strategy, middlewareData };
}

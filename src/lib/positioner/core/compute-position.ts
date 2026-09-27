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
 * Compute the floating element's final coordinates relative to the reference.
 * The reference can be a real DOM element or a virtual reference element (see
 * createVirtualElement); the latter typically anchors to a coordinate, such
 * as a context menu's mouse click position.
 * Middleware run in order; any middleware returning reset reruns the loop with
 * the new placement, at most MAX_MIDDLEWARE_PASSES times to avoid a deadlock.
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

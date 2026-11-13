import { createFloatingMiddleware, type Middleware } from "~/lib";

export {
  toPhysicalSide,
  toPlacement,
  getTransformOrigin,
} from "~/lib";

export interface BuildTooltipMiddlewareOptions {
  sideOffset: number;

  alignOffset: number;
  collisionPadding: number;
  arrowElement: () => Element | undefined;
}

export function createTooltipMiddleware(
  options: BuildTooltipMiddlewareOptions,
): Middleware[] {
  return createFloatingMiddleware({
    sideOffset: options.sideOffset,
    alignOffset: options.alignOffset,
    collisionPadding: options.collisionPadding,
    arrowElement: options.arrowElement,
    arrowPadding: 6,
  });
}

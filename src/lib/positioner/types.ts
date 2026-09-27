export type Side = "top" | "right" | "bottom" | "left";
export type Alignment = "start" | "end";
export type AlignedPlacement = `${Side}-${Alignment}`;
export type Placement = Side | AlignedPlacement;
export type Strategy = "absolute" | "fixed";
export type Axis = "x" | "y";

export interface Coords {
  x: number;
  y: number;
}

export interface Rect extends Coords {
  width: number;
  height: number;
}

export interface ElementRects {
  reference: Rect;
  floating: Rect;
}

export interface Elements {
  reference: ReferenceElement;
  floating: HTMLElement;
}

/**
 * Virtual reference element: anchors to a position where no real DOM node
 * exists — the typical case is a context menu, where the anchor is the mouse
 * click's pixel coordinate, not an element. Implementing getBoundingClientRect()
 * lets it take part in positioning like a real HTMLElement.
 */
export interface VirtualElement {
  getBoundingClientRect(): Rect;
  /**
   * Optional: associate a real DOM node; autoUpdate uses it to find scrollable
   * ancestor containers (the virtual element has no parent chain of its own,
   * so there is no way to tell which ancestors' scrolling affects this point).
   */
  contextElement?: Element;
}

export type ReferenceElement = Element | VirtualElement;

export interface MiddlewareData {
  [key: string]: any;
}

/** The current computation state passed to each middleware */
export interface MiddlewareState extends Coords {
  initialPlacement: Placement;
  placement: Placement;
  strategy: Strategy;
  rects: ElementRects;
  elements: Elements;
  middlewareData: MiddlewareData;
}

export interface MiddlewareReturn extends Partial<Coords> {
  data?: any;
  /**
   * If this field is returned, another pass must be computed with the new
   * placement; the main loop reruns once with it (used by flip and friends).
   */
  reset?: boolean | { placement?: Placement };
}

export interface Middleware {
  name: string;
  fn(state: MiddlewareState): MiddlewareReturn;
}

export interface ComputePositionConfig {
  placement?: Placement;
  strategy?: Strategy;
  middleware?: Middleware[];
}

export interface ComputePositionReturn extends Coords {
  placement: Placement;
  strategy: Strategy;
  middlewareData: MiddlewareData;
}

export interface Boundary extends Rect {}

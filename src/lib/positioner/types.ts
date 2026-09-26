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
 * 虚拟参照元素：用于锚定到一个不存在真实 DOM 节点的位置，
 * 最典型的场景就是右键菜单——锚点是鼠标点击的那个像素坐标，不是某个元素。
 * 只要实现 getBoundingClientRect() 就能像真实 HTMLElement 一样参与定位计算。
 */
export interface VirtualElement {
  getBoundingClientRect(): Rect;
  /**
   * 可选：关联一个真实 DOM 节点，autoUpdate 会用它去查找可滚动祖先容器
   * （虚拟元素本身没有父节点链，无法直接判断哪些祖先滚动会影响这个点的位置）。
   */
  contextElement?: Element;
}

export type ReferenceElement = Element | VirtualElement;

export interface MiddlewareData {
  [key: string]: any;
}

/** 传给每个 middleware 的当前计算状态 */
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
   * 若返回该字段，表示需要用新的 placement 重新计算一轮，
   * 主循环会用新 placement 重跑一次（用于 flip 等场景）。
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

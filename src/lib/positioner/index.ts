export { createPositioner } from "./create-positioner";
export type { CreatePositionerOptions, Positioner } from "./create-positioner";

export { computePosition } from "./core/compute-position";
export {
  getSide,
  getAlignment,
  getOppositePlacement,
  getOppositeAlignmentPlacement,
  ALL_PLACEMENTS,
} from "./core/placement";

export { autoUpdate } from "./auto-update";
export type { AutoUpdateOptions } from "./auto-update";

export { createVirtualElement } from "./virtual-element";
export type { PointOptions } from "./virtual-element";

export {
  offset,
  shift,
  flip,
  arrow,
  size,
  hide,
  containingBlockOffset,
} from "./middleware";
export type {
  OffsetOptions,
  OffsetValue,
  ShiftOptions,
  FlipOptions,
  ArrowOptions,
  ArrowData,
  SizeOptions,
  SizeAvailableSpace,
  HideOptions,
  HideData,
} from "./middleware";

export type {
  Placement,
  Side,
  Alignment,
  AlignedPlacement,
  Strategy,
  Axis,
  Coords,
  Rect,
  ElementRects,
  Elements,
  ReferenceElement,
  VirtualElement,
  Middleware,
  MiddlewareData,
  MiddlewareState,
  MiddlewareReturn,
  ComputePositionConfig,
  ComputePositionReturn,
  Boundary,
} from "./types";

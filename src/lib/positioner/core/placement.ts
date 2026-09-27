import type {
  Alignment,
  ElementRects,
  Placement,
  Side,
  Coords,
} from "../types";

export function getSide(placement: Placement): Side {
  return placement.split("-")[0] as Side;
}

export function getAlignment(placement: Placement): Alignment | undefined {
  return placement.split("-")[1] as Alignment | undefined;
}

export function isVerticalSide(side: Side): boolean {
  return side === "top" || side === "bottom";
}

export function getOppositeSide(side: Side): Side {
  const map: Record<Side, Side> = {
    top: "bottom",
    bottom: "top",
    left: "right",
    right: "left",
  };
  return map[side];
}

export function getOppositePlacement(placement: Placement): Placement {
  const side = getSide(placement);
  const alignment = getAlignment(placement);
  const opposite = getOppositeSide(side);
  return (alignment ? `${opposite}-${alignment}` : opposite) as Placement;
}

/** On the same side, swap start <-> end (used to flip alignment when still overflowing after shift) */
export function getOppositeAlignmentPlacement(placement: Placement): Placement {
  const side = getSide(placement);
  const alignment = getAlignment(placement);
  if (!alignment) return placement;
  const opposite = alignment === "start" ? "end" : "start";
  return `${side}-${opposite}` as Placement;
}

/**
 * Compute the floating element's top-left coordinates from the reference /
 * floating rects and the placement.
 * Note: this only computes the "ideal coordinates" — no viewport clipping;
 * clipping is left to the shift middleware.
 */
export function computeCoordsFromPlacement(
  rects: ElementRects,
  placement: Placement,
): Coords {
  const { reference, floating } = rects;
  const side = getSide(placement);
  const alignment = getAlignment(placement);
  const vertical = isVerticalSide(side);

  const centerX = reference.x + reference.width / 2 - floating.width / 2;
  const centerY = reference.y + reference.height / 2 - floating.height / 2;

  let x: number;
  let y: number;

  switch (side) {
    case "top":
      x = centerX;
      y = reference.y - floating.height;
      break;
    case "bottom":
      x = centerX;
      y = reference.y + reference.height;
      break;
    case "left":
      x = reference.x - floating.width;
      y = centerY;
      break;
    case "right":
      x = reference.x + reference.width;
      y = centerY;
      break;
  }

  if (alignment === "start") {
    if (vertical) x = reference.x;
    else y = reference.y;
  } else if (alignment === "end") {
    if (vertical) x = reference.x + reference.width - floating.width;
    else y = reference.y + reference.height - floating.height;
  }

  return { x, y };
}

export const ALL_PLACEMENTS: Placement[] = [
  "top",
  "top-start",
  "top-end",
  "right",
  "right-start",
  "right-end",
  "bottom",
  "bottom-start",
  "bottom-end",
  "left",
  "left-start",
  "left-end",
];

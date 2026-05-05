import type { SwipeDirection, ToasterProps } from "./types";
import { MOBILE_VIEWPORT_OFFSET, VIEWPORT_OFFSET } from "./constants";

/** Derives default swipe directions from a position string like "bottom-right" */
export function getDefaultSwipeDirections(position: string): SwipeDirection[] {
  const [y, x] = position.split("-");
  const directions: SwipeDirection[] = [];
  if (y) directions.push(y as SwipeDirection);
  if (x) directions.push(x as SwipeDirection);
  return directions;
}

/** Reads the document text direction (for RTL support) */
export function getDocumentDirection(): ToasterProps["dir"] {
  if (typeof window === "undefined") return "ltr";
  if (typeof document === "undefined") return "ltr";
  const dir = document.documentElement.getAttribute("dir");
  if (dir === "auto" || !dir) {
    return window.getComputedStyle(document.documentElement)
      .direction as ToasterProps["dir"];
  }
  return dir as ToasterProps["dir"];
}

/**
 * Converts offset prop (string | number | object) into CSS custom property map.
 * Produces both desktop (`--offset-*`) and mobile (`--mobile-offset-*`) vars.
 */
export function assignOffset(
  defaultOffset: ToasterProps["offset"],
  mobileOffset: ToasterProps["mobileOffset"],
): Record<string, string> {
  const styles: Record<string, string> = {};

  [defaultOffset, mobileOffset].forEach((offset, i) => {
    const isMobile = i === 1;
    const prefix = isMobile ? "--mobile-offset" : "--offset";
    const fallback = isMobile ? MOBILE_VIEWPORT_OFFSET : VIEWPORT_OFFSET;

    const assignAll = (value: string | number) => {
      ["top", "right", "bottom", "left"].forEach((side) => {
        styles[`${prefix}-${side}`] =
          typeof value === "number" ? `${value}px` : value;
      });
    };

    if (typeof offset === "number" || typeof offset === "string") {
      assignAll(offset);
    } else if (typeof offset === "object" && offset !== null) {
      ["top", "right", "bottom", "left"].forEach((side) => {
        const val = (offset as Record<string, string | number | undefined>)[
          side
        ];
        styles[`${prefix}-${side}`] =
          val === undefined
            ? fallback
            : typeof val === "number"
              ? `${val}px`
              : val;
      });
    } else {
      assignAll(fallback);
    }
  });

  return styles;
}

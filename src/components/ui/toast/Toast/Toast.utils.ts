import type { JSX } from "solid-js";
import {
  type Action,
  isAction,
  type Position,
  type ToastT,
} from "./Toast.types";

export function getToastAction(value: ToastT["action"]): Action | undefined {
  return isAction(value) ? value : undefined;
}

export function resolveToastContent(value: ToastT["title"]): JSX.Element {
  return typeof value === "function" ? value() : value;
}

export function getVerticalAxis(position: Position): "top" | "bottom" {
  return position.startsWith("top") ? "top" : "bottom";
}

export function isFrontToast(index: number): boolean {
  return index === 0;
}

export function getCollapsedTransform(options: {
  index: number;
  gap: number;
  position: Position;
}): string | undefined {
  const { index, gap, position } = options;
  if (isFrontToast(index)) return undefined;

  const lift = getVerticalAxis(position) === "bottom" ? -1 : 1;
  const scale = Math.max(0.8, 1 - index * 0.05);
  return `translateY(calc(${lift * gap * index}px)) scale(${scale})`;
}

export function getToastStyle(options: {
  toast: ToastT;
  index: number;
  total: number;
  expanded: boolean;
  position: Position;
  gap: number;
}): JSX.CSSProperties {
  const { toast, index, total, expanded, position, gap } = options;

  return {
    ...toast.style,

    "z-index": total - index,
    ...(expanded
      ? {}
      : {
          "grid-area": "1 / 1",
          "align-self":
            getVerticalAxis(position) === "bottom" ? "end" : "start",
          transform: getCollapsedTransform({ index, gap, position }),
          "pointer-events": isFrontToast(index) ? "auto" : "none",
        }),
  } as JSX.CSSProperties;
}

export function getAnimationClasses(options: {
  index: number;
  expanded: boolean;
  position: Position;
}): string {
  const { index, expanded, position } = options;
  if (!isFrontToast(index) && !expanded) return "";

  const isBottom = getVerticalAxis(position) === "bottom";
  const slideIn = isBottom
    ? "data-[state=open]:slide-in-from-bottom-2"
    : "data-[state=open]:slide-in-from-top-2";
  const slideOut = isBottom
    ? "data-[state=closed]:slide-out-to-bottom"
    : "data-[state=closed]:slide-out-to-top";

  return `${slideIn} ${slideOut} data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:fill-mode-both data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:fill-mode-both`;
}

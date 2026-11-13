import {
  offset,
  flip,
  shift,
  size,
  hide,
  containingBlockOffset,
  type Middleware,
} from "~/lib";
import { getViewportBoundary } from "~/lib/positioner";
import type { SelectOptionValue } from "../Select/Select.types";

function matchReferenceWidth(): Middleware {
  return {
    name: "matchWidth",
    fn(state) {
      return { data: { width: state.rects.reference.width } };
    },
  };
}

function alignSelectedItem(
  getValue: () => SelectOptionValue | null | undefined,
  getScrollElement: () => HTMLElement | undefined,
): Middleware {
  return {
    name: "itemAlign",
    fn(state) {
      const value = getValue();
      const scrollEl = getScrollElement();
      if (value == null || !scrollEl) return {};

      const itemEl = scrollEl.querySelector<HTMLElement>(
        `[data-value="${CSS.escape(String(value))}"]`,
      );
      if (!itemEl) return {};

      const { rects, strategy } = state;
      const boundary = getViewportBoundary(strategy, 8);

      const itemOffsetTop = itemEl.offsetTop;
      const itemHeight = itemEl.offsetHeight;
      const scrollHeight = scrollEl.scrollHeight;

      const idealItemTop =
        rects.reference.y + rects.reference.height / 2 - itemHeight / 2;

      const contentAbove = itemOffsetTop;
      const contentBelow = scrollHeight - itemOffsetTop - itemHeight;
      const spaceAbove = idealItemTop - boundary.y;
      const spaceBelow =
        boundary.y + boundary.height - (idealItemTop + itemHeight);

      const expandAbove = Math.max(0, Math.min(contentAbove, spaceAbove));
      const expandBelow = Math.max(0, Math.min(contentBelow, spaceBelow));

      const panelTop = idealItemTop - expandAbove;

      const scrollStyle = getComputedStyle(scrollEl);
      const borderTop = Number.parseFloat(scrollStyle.borderTopWidth) || 0;
      const borderBottom =
        Number.parseFloat(scrollStyle.borderBottomWidth) || 0;
      const panelHeight =
        expandAbove + itemHeight + expandBelow + borderTop + borderBottom;
      const scrollTop = contentAbove - expandAbove;

      scrollEl.scrollTop = scrollTop;

      return {
        x: rects.reference.x,
        y: panelTop,
        data: { aligned: true, panelHeight },
      };
    },
  };
}

export interface BuildSelectMiddlewareOptions {
  useItemAlign: boolean;
  selectedValue: () => SelectOptionValue | null | undefined;

  scrollElement: () => HTMLElement | undefined;

  sideOffset: number;

  alignOffset: number;
  collisionPadding: number;
  onAvailableHeightChange: (availableHeight: number) => void;
}

export function createSelectMiddleware(
  options: BuildSelectMiddlewareOptions,
): Middleware[] {
  if (options.useItemAlign) {
    return [
      matchReferenceWidth(),

      alignSelectedItem(options.selectedValue, options.scrollElement),
      shift({
        padding: options.collisionPadding,
        mainAxis: false,
        crossAxis: true,
      }),
      hide(),
      containingBlockOffset(),
    ];
  }

  return [
    offset({
      mainAxis: options.sideOffset,
      crossAxis: options.alignOffset,
    }),
    flip(),
    shift({ padding: options.collisionPadding }),
    matchReferenceWidth(),
    size({
      padding: options.collisionPadding,
      apply({ availableHeight }) {
        options.onAvailableHeightChange(availableHeight);
      },
    }),
    hide(),
    containingBlockOffset(),
  ];
}

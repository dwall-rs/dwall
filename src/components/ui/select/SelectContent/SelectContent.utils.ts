import {
  offset,
  flip,
  shift,
  size,
  hide,
  containingBlockOffset,
  type Middleware,
  type Placement,
} from "~/lib";
import { getViewportBoundary } from "~/lib/positioner/utils/dom";
import type { SelectOptionValue } from "../Select/Select.types";

/** Match the overlay width to the trigger — no coordinate changes, it just passes the reference width up to be used as style.width */
function matchReferenceWidth(): Middleware {
  return {
    name: "matchWidth",
    fn(state) {
      return { data: { width: state.rects.reference.width } };
    },
  };
}

/**
 * "Selected-item alignment": make the selected item cover exactly the area
 * above the trigger — the classic effect of a native <select>/shadcn Select
 * when reopened. It is a completely different positioning strategy from
 * offset+flip+shift, so it gets its own middleware instead of reusing
 * shift/flip.
 *
 * The core idea is to compute "panel height, panel position, list scroll
 * offset" together, rather than assuming a panel height first and clamping
 * scrollTop after computing the position — the latter was tried twice and
 * both versions left problems (a gap between panel and trigger, or the
 * selected item silently clamped out of range by scrollTop and pushed to the
 * panel edge).
 *
 * Concrete algorithm: the selected item ideally sits vertically centered on
 * the trigger. Using that position as the baseline, measure how much can be
 * opened up "above" and "below":
 * - expandable above = min(content remaining above the selected item,
 *   space remaining above in the viewport)
 * - expandable below = min(content remaining below the selected item,
 *   space remaining below in the viewport)
 * Panel height is "expand above + selected item height + expand below"; the
 * panel's top position and scroll offset both fall straight out of these two
 * expansion amounts, with no extra clamping — because:
 * - an expansion amount never exceeds "how much content remains", so
 *   scrollTop always lands in a valid range;
 * - an expansion amount never exceeds "how much space remains in the
 *   viewport", so the panel never leaves the viewport;
 * - the selected item hugging the trigger is the premise of this algorithm
 *   and holds naturally, so there is no separate "should it hug the
 *   trigger" check.
 * Items at either end of the list (first/last) degrade automatically to the
 * correct behavior: e.g. with the first item selected, "content above" is 0,
 * so expand-above is always 0 and the panel top lands exactly where the
 * selected item belongs, reserving no extra space above.
 */
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

      // Where the selected item should ideally land: vertically centered on the trigger
      const idealItemTop =
        rects.reference.y + rects.reference.height / 2 - itemHeight / 2;

      const contentAbove = itemOffsetTop; // content remaining above the selected item (px)
      const contentBelow = scrollHeight - itemOffsetTop - itemHeight; // content remaining below
      const spaceAbove = idealItemTop - boundary.y; // space available from the ideal position to the viewport top
      const spaceBelow =
        boundary.y + boundary.height - (idealItemTop + itemHeight); // space available down to the viewport bottom

      const expandAbove = Math.max(0, Math.min(contentAbove, spaceAbove));
      const expandBelow = Math.max(0, Math.min(contentBelow, spaceBelow));

      const panelTop = idealItemTop - expandAbove;
      // panelHeight eventually lands on scrollEl as the CSS max-height.
      // scrollEl has a border (the class list includes border), and Tailwind
      // preflight sets box-sizing to border-box, so max-height constrains the
      // border-box height; but scrollHeight only contains content + padding,
      // not the border. Using scrollHeight directly as max-height would make
      // clientHeight shorter than scrollHeight by the top and bottom border
      // widths, so even when the list just barely fits it would overflow by
      // 1~2px, causing overflow:auto to paint a vertical scrollbar. The
      // border width is added back here so the border-box max-height can
      // fully contain all content.
      const scrollStyle = getComputedStyle(scrollEl);
      const borderTop = Number.parseFloat(scrollStyle.borderTopWidth) || 0;
      const borderBottom =
        Number.parseFloat(scrollStyle.borderBottomWidth) || 0;
      const panelHeight =
        expandAbove + itemHeight + expandBelow + borderTop + borderBottom;
      const scrollTop = contentAbove - expandAbove;

      // scrollTop is a pure DOM side effect that does not affect the coordinate computation itself, so setting it directly here is safe.
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
  /** Whether a value is already selected — decides between "selected-item alignment" and the plain edge-anchored dropdown */
  hasValue: boolean;
  selectedValue: () => SelectOptionValue | null | undefined;
  /** The element that actually scrolls and hosts the option list (inner layer), not the outer positioning container */
  scrollElement: () => HTMLElement | undefined;
  placement: Placement;
  collisionPadding: number;
  onAvailableHeightChange: (availableHeight: number) => void;
}

/**
 * Positioning pipeline for the Select panel, with two recipes depending on
 * whether there is a selected value:
 *
 * - **With a value**: matchWidth → size (computes the available height so
 *   overflow:auto can scroll — unlike Tooltip, dropdown content is naturally
 *   possibly long, so scrolling is a legitimate need, not over-engineering)
 *   → itemAlign (the core: cover the trigger with the selected item) → shift
 *   (horizontal only; vertical has already been clamped by itemAlign itself)
 *   → containingBlockOffset.
 * - **Without a value** (placeholder state): offset → flip → shift →
 *   matchWidth → size → containingBlockOffset. This is the standard
 *   "edge-anchored dropdown" strategy, the same idea Dropdown uses.
 *
 * The order must not be shuffled: size must come after shift/itemAlign (it
 * needs the final coordinates to back out the available height), and
 * containingBlockOffset must come after every other middleware.
 *
 * SelectContent.tsx splits the panel into two layers: the outer
 * (position:fixed, responsible for translate(x,y) positioning) and the inner
 * (real styling, overflow:auto scrolling, data-state-driven enter/exit
 * animation). This split is not arbitrary: if one element carried both the
 * positioning transform:translate() and a transform:scale()-based animation
 * like zoom-in-95, CSS allows only one transform per element, and the
 * animation would take the property over completely while playing, wiping
 * out the positioning translate(x,y) — producing the obviously wrong look of
 * "flying in from the viewport's top-left on open and flying back to the
 * top-left to disappear on close" (a real bug an early implementation of
 * this component hit). scrollElement receives the inner layer (the one that
 * actually scrolls), not state.elements.floating (the outer layer, which
 * does not scroll itself).
 */
export function createSelectMiddleware(
  options: BuildSelectMiddlewareOptions,
): Middleware[] {
  if (options.hasValue) {
    return [
      matchReferenceWidth(),
      // Note there is no size middleware here: alignSelectedItem has already
      // computed panel height, position, and scroll offset together (see its
      // comment), and the panel height is passed out via
      // middlewareData.itemAlign.panelHeight. size does not need to redo the
      // "viewport space only, content length ignored" computation
      // independently — if the two ever disagree, the CSS max-height and the
      // panel height assumed by the positioning calculation would fall out of
      // sync, reintroducing problems like "the selected item is not aligned"
      // (which is exactly the root of the bug fixed here).
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
    offset(4),
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

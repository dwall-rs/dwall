export interface RovingNavItem<TValue> {
  value: TValue;
  element: HTMLElement;

  disabled: () => boolean;
}

export interface RovingNavOptions<TValue> {
  getItems: () => RovingNavItem<TValue>[];

  orientation: () => "horizontal" | "vertical";

  dir: () => "ltr" | "rtl" | "auto" | undefined;

  loop: () => boolean;

  disabled?: () => boolean;

  getDirectionElement?: (event: KeyboardEvent) => Element | null;

  onHighlight: (value: TValue) => void;
}

export const ROVING_NAV_KEYS = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

function resolveRTL(
  dir: "ltr" | "rtl" | "auto" | undefined,
  event: KeyboardEvent,
  getDirectionElement: ((event: KeyboardEvent) => Element | null) | undefined,
): boolean {
  if (dir === "rtl") return true;
  if (dir === "ltr") return false;

  const target = getDirectionElement?.(event);
  if (target) {
    return getComputedStyle(target).direction === "rtl";
  }
  return (
    typeof document !== "undefined" && document.documentElement.dir === "rtl"
  );
}

export function computeNextIndex(
  key: string,
  currentIndex: number,
  itemCount: number,
  opts: { isVertical: boolean; isRTL: boolean; loop: boolean },
): number | undefined {
  let next: number | undefined;

  switch (key) {
    case "Home":
      next = 0;
      break;
    case "End":
      next = itemCount - 1;
      break;
    case "ArrowLeft":
      if (opts.isVertical) return undefined;
      next = currentIndex + (opts.isRTL ? 1 : -1);
      break;
    case "ArrowRight":
      if (opts.isVertical) return undefined;
      next = currentIndex + (opts.isRTL ? -1 : 1);
      break;
    case "ArrowUp":
      if (!opts.isVertical) return undefined;
      next = currentIndex - 1;
      break;
    case "ArrowDown":
      if (!opts.isVertical) return undefined;
      next = currentIndex + 1;
      break;
    default:
      return undefined;
  }

  if (opts.loop) {
    return (next + itemCount) % itemCount;
  }
  return Math.min(itemCount - 1, Math.max(0, next));
}

export function createRovingNavigation<TValue>(
  options: RovingNavOptions<TValue>,
) {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (options.disabled?.()) return;
    if (!ROVING_NAV_KEYS.has(event.key)) return;

    const activeEl = document.activeElement as HTMLElement | null;

    const items = options.getItems().filter((item) => !item.disabled());
    if (items.length === 0) return;

    const currentIndex = items.findIndex((item) => item.element === activeEl);
    if (currentIndex === -1) return;

    const isVertical = options.orientation() === "vertical";
    const isRTL = resolveRTL(options.dir(), event, options.getDirectionElement);

    const nextIndex = computeNextIndex(event.key, currentIndex, items.length, {
      isVertical,
      isRTL,
      loop: options.loop(),
    });
    if (nextIndex === undefined) return;

    event.preventDefault();

    const nextItem = items[nextIndex];
    nextItem.element.focus();
    options.onHighlight(nextItem.value);
  };

  return { handleKeyDown };
}

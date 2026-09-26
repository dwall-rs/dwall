import type {
  ToggleGroupContextValue,
  ToggleGroupValue,
} from "./ToggleGroup.types";

/** 键盘导航按键:方向键 + Home/End(对齐 base-ui CompositeRoot 的 enableHomeAndEndKeys) */
const NAV_KEYS = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

/**
 * ToggleGroup 的 roving focus 键盘导航(纯逻辑,可独立单测)。
 *
 * 方向映射:
 * - horizontal + ltr: ArrowRight → next, ArrowLeft → prev
 * - horizontal + rtl: 左右反转
 * - vertical:          ArrowDown → next, ArrowUp → prev
 * - Home → 第一个,End → 最后一个
 * - loopFocus=true 环绕,false 边界停止;跳过 disabled 的 item
 *
 * 只有焦点已经在某个 item 上时才接管按键,避免影响外部其它元素;
 * 移动焦点后由 item 的 onFocus 更新高亮。
 */
export function useToggleGroupKeyboard<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(ctx: ToggleGroupContextValue<TValue>) {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (ctx.disabled()) return;
    if (!NAV_KEYS.has(e.key)) return;

    const activeEl = document.activeElement as HTMLElement | null;
    const items = ctx.getItems().filter((item) => !item.disabled());
    if (items.length === 0) return;

    const currentIndex = items.findIndex((item) => item.element === activeEl);
    if (currentIndex === -1) return;

    const isVertical = ctx.orientation() === "vertical";
    // dir 未指定/auto 时,按根元素的实际书写方向判断(兼容祖先节点上的 dir)
    const dir = ctx.dir();
    const isRTL =
      dir === "rtl" ||
      (dir !== "ltr" &&
        typeof document !== "undefined" &&
        getComputedStyle(e.currentTarget as Element).direction === "rtl");

    let nextIndex: number | undefined;
    switch (e.key) {
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = items.length - 1;
        break;
      case "ArrowLeft":
        if (isVertical) return;
        nextIndex = currentIndex + (isRTL ? 1 : -1);
        break;
      case "ArrowRight":
        if (isVertical) return;
        nextIndex = currentIndex + (isRTL ? -1 : 1);
        break;
      case "ArrowUp":
        if (!isVertical) return;
        nextIndex = currentIndex - 1;
        break;
      case "ArrowDown":
        if (!isVertical) return;
        nextIndex = currentIndex + 1;
        break;
    }

    if (nextIndex === undefined) return;

    // 方向键与当前布局匹配,接管默认滚动行为
    e.preventDefault();

    if (ctx.loopFocus()) {
      nextIndex = (nextIndex + items.length) % items.length;
    } else {
      nextIndex = Math.min(items.length - 1, Math.max(0, nextIndex));
    }

    items[nextIndex].element.focus();
    ctx.setHighlightedValue(items[nextIndex].value);
  };

  return { handleKeyDown };
}

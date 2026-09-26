import type { TabsContextValue } from "./Tabs.context";

const NAV_KEYS = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

/**
 * tablist 的 roving focus 键盘导航(纯逻辑,可在事件中独立单测)
 *
 * 方向映射:
 * - horizontal + ltr: ArrowRight → next, ArrowLeft → prev
 * - horizontal + rtl: 左右反转
 * - vertical:          ArrowDown → next, ArrowUp → prev
 * - Home → 第一个,End → 最后一个
 * - loop=true 环绕,false 边界停止;跳过 disabled 的 trigger
 *
 * 与 base-ui 一致:方向键只移动焦点(触发 onFocus 更新高亮),不激活;
 * 激活靠点击(Enter/Space 触发 button 的 click)。activateOnFocus 时的
 * 聚焦激活由 TabsTrigger 的 onFocus 处理,这里不做。
 */
export function useTabsKeyboard(ctx: TabsContextValue) {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!NAV_KEYS.has(e.key)) return;

    const activeEl = document.activeElement as HTMLElement | null;
    // 只有焦点在 tab 上时才接管键盘
    if (activeEl?.getAttribute("role") !== "tab") return;

    const tabs = ctx.getTriggers().filter((t) => !t.disabled());
    if (tabs.length === 0) return;

    const currentIndex = tabs.findIndex((t) => t.element === activeEl);
    if (currentIndex === -1) return;

    const isVertical = ctx.orientation() === "vertical";
    // dir=auto 时回退到文档实际方向
    const dir = ctx.dir();
    const isRTL =
      dir === "rtl" ||
      (dir === "auto" &&
        typeof document !== "undefined" &&
        document.documentElement.dir === "rtl");

    let nextIndex: number | undefined;
    switch (e.key) {
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = tabs.length - 1;
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

    if (ctx.loop()) {
      nextIndex = (nextIndex + tabs.length) % tabs.length;
    } else {
      nextIndex = Math.min(tabs.length - 1, Math.max(0, nextIndex));
    }

    // focus 触发 trigger 的 onFocus → 更新高亮(及 activateOnFocus 时激活)
    tabs[nextIndex].element.focus();
  };

  return { handleKeyDown };
}

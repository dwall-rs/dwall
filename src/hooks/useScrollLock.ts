import { createEffect, onCleanup, type Accessor } from "solid-js";

export interface UseScrollLockOptions {
  /**
   * 允许在锁滚动期间继续滚动的容器选择器(通常是浮层自身)。
   * 未命中该选择器的滚轮/触摸滚动会被拦截;不传则整个页面都锁定滚动。
   */
  allowedSelector?: string;
}

// 多个浮层/嵌套浮层可能同时持锁,用计数避免其中一个关闭就提前恢复页面滚动。
let lockCount = 0;
let release: (() => void) | undefined;
// 各持锁浮层允许滚动的容器选择器。preventScroll 读取的是实时集合,
// 因此即使某个选择器在锁生效之后才加入,也能立即对它放行。
const allowedSelectors = new Set<string>();

/** 元素自身是否建立了滚动容器(与 positioner 里的判断口径一致) */
function isOverflowElement(el: Element): boolean {
  const { overflow, overflowX, overflowY } = getComputedStyle(el);
  return /auto|scroll|overlay|hidden/.test(overflow + overflowX + overflowY);
}

/**
 * 页面的滚动容器:html 自己建立了滚动上下文时锁 html,否则锁 body
 * ——把 overflow 写在另一个元素上并不会锁住页面(Base UI 同样的判断)。
 */
function getViewportScroller(): HTMLElement {
  const html = document.documentElement;
  return isOverflowElement(html) ? html : document.body;
}

function supportsStableScrollbarGutter(): boolean {
  return (
    typeof CSS !== "undefined" &&
    typeof CSS.supports === "function" &&
    CSS.supports("scrollbar-gutter", "stable")
  );
}

function isWithinAllowedScroll(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  for (const selector of allowedSelectors) {
    if (target.closest(selector)) return true;
  }
  return false;
}

function lockViewport(): () => void {
  const html = document.documentElement;
  const body = document.body;
  const scroller = getViewportScroller();

  const previous = {
    htmlScrollbarGutter: html.style.scrollbarGutter,
    scrollerOverflowX: scroller.style.overflowX,
    scrollerOverflowY: scroller.style.overflowY,
    bodyPaddingRight: body.style.paddingRight,
    scrollTop: scroller.scrollTop,
    scrollLeft: scroller.scrollLeft,
  };

  // 抵消滚动条消失带来的横向抖动:能用 scrollbar-gutter 就用它,
  // 否则退化成给 body 补一个滚动条宽度的右内边距。
  // 只有页面自身真的存在占位滚动条时才需要补偿:滚动条长在某个嵌套容器上时,
  // 锁 html/body 并不会移除那条滚动条,再给 html 加 gutter 只会额外多出一条空白
  // (滚动条 + gutter 双倍宽度),反而破坏布局。
  const scrollbarWidth = Math.max(0, window.innerWidth - html.clientWidth);
  const gutterSupported = supportsStableScrollbarGutter();
  if (scrollbarWidth > 0) {
    if (gutterSupported) {
      html.style.scrollbarGutter = "stable";
    } else {
      const padding =
        Number.parseFloat(getComputedStyle(body).paddingRight) || 0;
      body.style.paddingRight = `${padding + scrollbarWidth}px`;
    }
  }

  scroller.style.overflowX = "hidden";
  scroller.style.overflowY = "hidden";

  // 兜底:部分浏览器即使 overflow:hidden 仍能被滚轮/触摸滚动,
  // 因此直接拦截浮层之外的滚动事件;浮层内部(命中 allowedSelector)保留滚动能力。
  const preventScroll = (event: Event) => {
    if (isWithinAllowedScroll(event.target)) return;
    event.preventDefault();
  };
  document.addEventListener("wheel", preventScroll, { passive: false });
  document.addEventListener("touchmove", preventScroll, { passive: false });

  return () => {
    html.style.scrollbarGutter = previous.htmlScrollbarGutter;
    if (!gutterSupported) {
      body.style.paddingRight = previous.bodyPaddingRight;
    }
    scroller.style.overflowX = previous.scrollerOverflowX;
    scroller.style.overflowY = previous.scrollerOverflowY;
    document.removeEventListener("wheel", preventScroll);
    document.removeEventListener("touchmove", preventScroll);
    scroller.scrollTop = previous.scrollTop;
    scroller.scrollLeft = previous.scrollLeft;
  };
}

function acquireLock(allowedSelector?: string): () => void {
  lockCount += 1;
  if (allowedSelector) allowedSelectors.add(allowedSelector);
  if (lockCount === 1) release = lockViewport();

  let released = false;
  return () => {
    if (released) return;
    released = true;
    lockCount -= 1;
    if (lockCount === 0) {
      release?.();
      release = undefined;
      // 全部释放后清空放行名单,避免残留的选择器影响下一次锁。
      allowedSelectors.clear();
    }
  };
}

/**
 * 浮层打开时锁定页面滚动,关闭后恢复——对齐 Base UI 的 modal 行为
 * (文档滚动被锁定、浮层之外的滚轮/触摸滚动被拦截),避免浮窗锚点随页面滚动而"飘移"。
 *
 * 多个浮层可同时持锁(引用计数);`allowedSelector` 命中的容器内仍可滚动,
 * 通常传入浮层自身,例如 Popover 传 `[data-slot="popover-content"]`。
 */
export function useScrollLock(
  locked: Accessor<boolean>,
  options?: UseScrollLockOptions,
) {
  createEffect(() => {
    if (!locked()) return;
    const unlock = acquireLock(options?.allowedSelector);
    onCleanup(unlock);
  });
}

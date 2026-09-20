import {
  Show,
  onCleanup,
  splitProps,
  createMemo,
  createEffect,
} from "solid-js";
import { Portal } from "solid-js/web";
import { useSelectContent } from "./useSelectContent";
import type { SelectContentProps } from "./SelectContent.types";

function cn(...classes: Array<string | false | undefined | null>): string {
  return classes.filter(Boolean).join(" ");
}

export function SelectContent(props: SelectContentProps) {
  const {
    ctx,
    pos,
    maxHeight,
    mounted,
    animationState,
    contentElement,
    setContentElement,
  } = useSelectContent(() => props);

  const [local, rest] = splitProps(props, [
    "placement",
    "collisionPadding",
    "class",
    "style",
    "children",
  ]);

  const enabledItems = createMemo(() => ctx.items.filter((it) => !it.disabled));

  const moveActive = (dir: 1 | -1) => {
    const list = enabledItems();
    if (list.length === 0) return;
    const currentIndex = list.findIndex((it) => it.value === ctx.activeValue());
    const nextIndex = (currentIndex + dir + list.length) % list.length;
    ctx.setActiveValue(list[nextIndex].value);
  };

  const onKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        moveActive(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        moveActive(-1);
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        if (ctx.activeValue() !== undefined)
          ctx.selectValue(ctx.activeValue()!);
        break;
      case "Escape":
        e.preventDefault();
        ctx.close();
        break;
      case "Tab":
        ctx.setOpen(false);
        break;
    }
  };

  // 点击外部关闭：捕获阶段监听 + stopPropagation，在事件还没下沉到真正的目标
  // 元素之前就拦截掉，避免点击穿透到浮层下面被盖住的元素（细节见
  // production-tooltip 那边点击穿透问题的排查记录，这里直接复用同一个模式）。
  createEffect(() => {
    if (!ctx.open()) return;
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as Node;
      if (
        !ctx.reference()?.contains(target) &&
        !ctx.floating()?.contains(target)
      ) {
        ctx.setOpen(false);
        e.stopPropagation();
      }
    };
    document.addEventListener("pointerdown", onPointerDown, { capture: true });
    onCleanup(() =>
      document.removeEventListener("pointerdown", onPointerDown, {
        capture: true,
      }),
    );
  });

  // 打开时把键盘事件绑到 document，这样不需要抢 focus 也能用方向键
  createEffect(() => {
    if (!ctx.open()) return;
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  // 高亮项变化时自动滚动到可见区域
  createEffect(() => {
    const active = ctx.activeValue();
    const el = contentElement();
    if (!active || !el) return;
    el.querySelector<HTMLElement>(
      `[data-value="${CSS.escape(active)}"]`,
    )?.scrollIntoView({ block: "nearest" });
  });

  return (
    <Show when={mounted()}>
      <Portal>
        {/*
          外层：只负责定位（position:fixed 来自 floatingStyles），不参与动画。
          它的 transform 专门用来做 translate(x,y) 定位——如果内层的 zoom-in-95
          动画也放在这一层，两者会互相冲突（细节见 SelectContent.utils.ts 顶部
          的注释）。
        */}
        <div
          ref={(el) => {
            ctx.setFloating(el);
            // 元素卸载时清空 ctx.floating()，避免后续读到已从 DOM 摘除的
            // 僵尸节点（Solid 的 ref 回调只在创建时触发一次，不会自动在
            // 卸载时传 undefined）。
            onCleanup(() => ctx.setFloating(undefined));
          }}
          style={{
            ...pos.floatingStyles(),
            "z-index": 1000,
            opacity: pos.isPositioned() ? 1 : 0,
          }}
        >
          {/*
            内层：真正的视觉样式 + overflow:auto 滚动 + 进出场动画。
            position: relative 是为了让它成为内部所有选项的 offsetParent——
            "选中项对齐"这套定位策略要用 offsetTop 算选项在列表里的位置，
            这个值必须相对"真正会滚动的这个元素"才准确，不能让它被外层
            （不滚动）抢走 offsetParent 的身份。
          */}
          <ul
            ref={setContentElement}
            role="listbox"
            data-state={animationState()}
            style={{
              position: "relative",
              width: pos.middlewareData().matchWidth?.width
                ? `${pos.middlewareData().matchWidth.width}px`
                : undefined,
              "max-height": maxHeight() ? `${maxHeight()}px` : undefined,
              overflow: "auto",
              ...(typeof local.style === "object" ? local.style : undefined),
            }}
            class={cn(
              "min-w-[8rem] rounded-md border border-neutral-200 bg-white p-1 shadow-md",
              "dark:border-neutral-800 dark:bg-neutral-950",
              // 只保留入场动画，没有退场动画：选中一项的瞬间 ctx.value() 变化，
              // 会让定位策略从"贴边下拉"切到"选中项对齐"，如果退场动画期间
              // 面板还挂在 DOM 里，这次重新定位会让面板先跳一下再消失，观感
              // 很突兀，去掉退场动画从根上避免这个问题（配合 useSelectContent.ts
              // 里 mounted 直接跟随 isVisible、不做 Presence 延迟卸载）。
              // duration-100（100ms）比默认的 150ms 更短——下拉菜单这种高频
              // 交互的组件，动画应该比 tooltip 更快、更利落。
              "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95 data-[state=open]:duration-100",
              local.class,
            )}
            {...rest}
          >
            {local.children}
          </ul>
        </div>
      </Portal>
    </Show>
  );
}

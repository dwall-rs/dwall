import { onCleanup, splitProps, createMemo, createEffect } from "solid-js";
import { Portal } from "solid-js/web";
import { useSelectContent } from "./useSelectContent";
import { ScrollArrows } from "~/components/ui/scroll-arrows";
import type { SelectContentProps } from "./SelectContent.types";
import { clsx } from "~/utils";
import { getAlignment, getSide } from "~/lib";

export function SelectContent(props: SelectContentProps) {
  const {
    ctx,
    pos,
    placement,
    aligned,
    maxHeight,
    isVisible,
    animationState,
    contentElement,
    setContentElement,
  } = useSelectContent(() => props);

  const [local, rest] = splitProps(props, [
    "side",
    "align",
    "sideOffset",
    "alignOffset",
    "alignItemWithTrigger",
    "placement",
    "collisionPadding",
    "class",
    "style",
    "children",
  ]);

  const dataAttrs = createMemo(() => ({
    "data-placement": placement(),
    "data-side": aligned() ? "none" : getSide(placement()),
    "data-align": aligned()
      ? undefined
      : (getAlignment(placement()) ?? "center"),
  }));

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

  createEffect(() => {
    if (!ctx.open()) return;
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  createEffect(() => {
    const active = ctx.activeValue();
    const el = contentElement();
    if (!active || !el) return;
    el.querySelector<HTMLElement>(
      `[data-value="${CSS.escape(String(active))}"]`,
    )?.scrollIntoView({ block: "nearest" });
  });

  return (
    <Portal>
      <div
        data-slot="select-content"
        {...dataAttrs()}
        ref={(el) => {
          ctx.setFloating(el);

          onCleanup(() => ctx.setFloating(undefined));
        }}
        style={{
          ...pos.floatingStyles(),
          "z-index": 1000,
          opacity: isVisible() && pos.isPositioned() ? 1 : 0,
          "pointer-events": isVisible() ? "auto" : "none",
          visibility: isVisible() ? "visible" : "hidden",
        }}
      >
        <div
          ref={setContentElement}
          role="listbox"
          data-state={animationState()}
          {...dataAttrs()}
          style={{
            position: "relative",
            width: pos.middlewareData().matchWidth?.width
              ? `${pos.middlewareData().matchWidth.width}px`
              : undefined,
            "max-height": maxHeight() ? `${maxHeight()}px` : undefined,
            overflow: "auto",
            ...(typeof local.style === "object" ? local.style : undefined),
          }}
          class={clsx(
            "min-w-36 max-w-[calc(100vw-1rem)] overscroll-contain",
            "no-scrollbar [&::-webkit-scrollbar]:hidden",
            "rounded-lg bg-popover text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100",

            "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
            local.class,
          )}
          {...rest}
        >
          {local.children}
        </div>
        <ScrollArrows
          target={contentElement}
          upClass="rounded-t-lg"
          downClass="rounded-b-lg"
        />
      </div>
    </Portal>
  );
}

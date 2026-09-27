import { onCleanup, splitProps, createMemo, createEffect } from "solid-js";
import { Portal } from "solid-js/web";
import { useSelectContent } from "./useSelectContent";
import { ScrollArrows } from "~/components/ui/scroll-arrows";
import type { SelectContentProps } from "./SelectContent.types";
import { clsx } from "~/utils";

export function SelectContent(props: SelectContentProps) {
  const {
    ctx,
    pos,
    maxHeight,
    isVisible,
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

  // Close on outside click: a capture-phase listener + stopPropagation
  // intercepts the event before it bubbles down to the real target element,
  // preventing clicks from falling through to elements covered underneath the
  // overlay (see the click-through investigation notes in production-tooltip;
  // the same pattern is reused here).
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

  // While open, bind keyboard events to document so arrow keys work without
  // stealing focus
  createEffect(() => {
    if (!ctx.open()) return;
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  // Scroll the highlighted item into view when it changes
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
      {/*
        Outer layer: handles positioning only (position:fixed comes from
        floatingStyles) and takes no part in the animation. Its transform is
        reserved for translate(x,y) positioning — putting the inner zoom-in-95
        animation on this layer as well would make the two conflict (see the
        comment at the top of SelectContent.utils.ts for details).

        On close the whole Portal is no longer unmounted; it is hidden via
        visibility/pointer-events instead: SelectItem registers its item
        metadata with ctx via onMount, and unmounting the children on close
        would clear ctx.items, leaving SelectValue unable to render the
        selected value from item.label. Keeping the children mounted makes the
        registration lifetime of items match SelectContent itself; hiding is
        only a visual/interactive close and does not affect data.
      */}
      <div
        data-slot="select-content"
        ref={(el) => {
          ctx.setFloating(el);
          // Clear ctx.floating() when the element unmounts so later reads
          // don't pick up a zombie node that has already been removed from
          // the DOM (Solid's ref callback fires only once on creation and
          // does not pass undefined automatically on unmount).
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
        {/*
            Inner layer: the real visual styling + overflow:auto scrolling +
            enter/exit animation. position: relative makes it the
            offsetParent of every option inside it — the "selected-item
            alignment" positioning strategy uses offsetTop to compute an
            option's position in the list, and that value is only accurate
            relative to "the element that actually scrolls"; the outer layer
            (which does not scroll) must not steal its offsetParent role.
          */}
        <div
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
          class={clsx(
            "min-w-36",
            "no-scrollbar [&::-webkit-scrollbar]:hidden",
            "rounded-lg bg-popover text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100",
            // Enter animation only, no exit animation: on close the outer
            // layer switches straight to visibility:hidden and data-state
            // flips back to closed immediately, so there is no exit phase and
            // no "panel jumps first, then disappears" effect when the
            // selection changes. duration-100 (100ms) is shorter than the
            // default 150ms — for a high-frequency component like a dropdown,
            // animations should be faster and snappier than a tooltip's.
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

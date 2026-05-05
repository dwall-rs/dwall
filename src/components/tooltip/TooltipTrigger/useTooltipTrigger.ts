import { onCleanup, onMount, type ChildrenReturn } from "solid-js";
import type { TimeoutID } from "~/types";
import { useTooltipContext } from "../Tooltip/Tooltip.context";

export const useTooltipTrigger = (resolved: ChildrenReturn) => {
  const { setOpen, openDelay, closeDelay, setTriggerRef } = useTooltipContext();

  let openTimer: TimeoutID;
  let closeTimer: TimeoutID;

  const open = () => {
    clearTimeout(closeTimer);
    openTimer = setTimeout(() => setOpen(true), openDelay);
  };

  const close = () => {
    clearTimeout(openTimer);
    closeTimer = setTimeout(() => setOpen(false), closeDelay);
  };

  const handleMouseEnter = (_: PointerEvent) => {
    open();
  };
  const handleMouseLeave = (_: PointerEvent | FocusEvent) => {
    close();
  };
  const handleMouseDown = (_: Event) => {
    close();
  };

  onMount(() => {
    const trigger = resolved.toArray();
    if (trigger.length !== 1) {
      console.warn("[Tooltip] TooltipTrigger should have exactly one child");
    }

    const el = trigger[0] as HTMLElement | undefined;

    if (!el) return;

    if (!(el instanceof HTMLElement)) {
      console.error("[Tooltip] TooltipTrigger child should be an HTMLElement");
      return;
    }

    setTriggerRef(el);
    el.addEventListener("pointerenter", handleMouseEnter);
    el.addEventListener("pointerleave", handleMouseLeave);
    el.addEventListener("mousedown", handleMouseDown);
    el.addEventListener("blur", handleMouseLeave);

    onCleanup(() => {
      clearTimeout(openTimer);
      clearTimeout(closeTimer);

      if (!el) return;
      el.removeEventListener("pointerenter", handleMouseEnter);
      el.removeEventListener("pointerleave", handleMouseLeave);
      el.removeEventListener("mousedown", handleMouseDown);
      el.removeEventListener("blur", handleMouseLeave);
    });
  });

  // createEffect(() => {
  //   el = resolved() as HTMLElement | undefined;
  //   if (!el) return;

  //   const handleMouseEnter = () => {
  //     open();
  //   };
  //   const handleMouseLeave = () => {
  //     close();
  //   };
  //   const handleMouseDown = () => close();

  //   untrack(() => setTriggerRef(el!));
  //   el.addEventListener("mouseenter", handleMouseEnter);
  //   el.addEventListener("mouseleave", handleMouseLeave);
  //   el.addEventListener("mousedown", handleMouseDown);
  //   el.addEventListener("blur", handleMouseLeave);

  //   onCleanup(() => {
  //     el!.removeEventListener("mouseenter", handleMouseEnter);
  //     el!.removeEventListener("mouseleave", handleMouseLeave);
  //     el!.removeEventListener("mousedown", handleMouseDown);
  //     el!.removeEventListener("blur", handleMouseLeave);
  //   });
  // });

  // onCleanup(() => {
  //   clearTimeout(openTimer);
  //   clearTimeout(closeTimer);
  // });
};

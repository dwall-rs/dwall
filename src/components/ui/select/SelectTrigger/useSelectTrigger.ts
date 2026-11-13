import { onCleanup, type ValidComponent } from "solid-js";
import { useSelectContext } from "../Select/Select.context";
import type { SelectTriggerProps } from "./SelectTrigger.types";

export interface UseSelectTriggerResult {
  ctx: ReturnType<typeof useSelectContext>;
  isDisabled: () => boolean;

  attachListeners: (el: Element) => void;
}

export function useSelectTrigger<T extends ValidComponent>(
  props: () => SelectTriggerProps<T>,
): UseSelectTriggerResult {
  const ctx = useSelectContext("SelectTrigger");

  const isDisabled = () => ctx.disabled() || !!props().disabled;

  const openAndHighlightSelected = () => {
    ctx.setActiveValue(
      ctx.value() ?? ctx.items.find((it) => !it.disabled)?.value,
    );
    ctx.setOpen(true);
  };

  const attachListeners = (el: Element) => {
    const onClick = () => {
      if (isDisabled()) return;
      if (ctx.open()) ctx.setOpen(false);
      else openAndHighlightSelected();
    };

    const onKeyDown = (e: Event) => {
      if (isDisabled()) return;
      const keyboardEvent = e as KeyboardEvent;

      if (!ctx.open()) {
        if (
          ["ArrowDown", "ArrowUp", "Enter", " "].includes(keyboardEvent.key)
        ) {
          keyboardEvent.preventDefault();
          openAndHighlightSelected();
        }
        return;
      }

      if (keyboardEvent.key === "Escape") {
        keyboardEvent.stopPropagation();
        ctx.close();
      }
    };

    el.addEventListener("click", onClick);
    el.addEventListener("keydown", onKeyDown);

    onCleanup(() => {
      el.removeEventListener("click", onClick);
      el.removeEventListener("keydown", onKeyDown);
    });
  };

  return { ctx, isDisabled, attachListeners };
}

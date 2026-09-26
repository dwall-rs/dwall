import { onCleanup, type ValidComponent } from "solid-js";
import { useSelectContext } from "../Select/Select.context";
import type { SelectTriggerProps } from "./SelectTrigger.types";

export interface UseSelectTriggerResult {
  ctx: ReturnType<typeof useSelectContext>;
  isDisabled: () => boolean;
  /**
   * 把开关下拉相关的原生事件监听器绑定到真正的 DOM 元素上（通过 ref 调用）。
   * 用 addEventListener 而不是 JSX 的 onXxx prop，是为了不占用这些 prop 名——
   * 调用方可以在 <SelectTrigger onClick={...}> 上自由传自己的原生事件处理，
   * 两者是完全独立的两套机制，互不覆盖。
   */
  attachListeners: (el: Element) => void;
}

export function useSelectTrigger<T extends ValidComponent>(
  props: () => SelectTriggerProps<T>,
): UseSelectTriggerResult {
  const ctx = useSelectContext("SelectTrigger");

  const isDisabled = () => ctx.disabled() || !!props().disabled;

  const openAndHighlightSelected = () => {
    if (isDisabled()) return;
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

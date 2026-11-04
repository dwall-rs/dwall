import { onCleanup, type ValidComponent } from "solid-js";
import { useDialogContext } from "../Dialog";
import type { DialogTriggerProps } from "./DialogTrigger.types";

export interface UseDialogTriggerResult {
  open: () => boolean;
  isDisabled: () => boolean;
  /**
   * 把打开 Dialog 的 click 监听器绑定到真正的 DOM 元素上（通过 ref 调用）。
   * 使用 addEventListener 而不是 JSX 的 onClick prop，是为了不占用 onClick——
   * 调用方自己传的 onClick 不会被覆盖，两者都会触发。
   */
  attachListeners: (el: Element) => void;
}

export function useDialogTrigger<T extends ValidComponent>(
  props: () => DialogTriggerProps<T>,
): UseDialogTriggerResult {
  const { open, setOpen, setTrigger } = useDialogContext();

  const isDisabled = () => !!props().disabled;

  const attachListeners = (el: Element) => {
    const onClick = () => {
      if (isDisabled()) return;
      if (open()) return;
      // 记下打开浮层的控件，关闭时焦点要回到它身上。鼠标点击与 Enter/空格
      // 都会让它成为 activeElement；取不到时退回触发元素本身。
      const active = document.activeElement;
      setTrigger(active instanceof HTMLElement ? active : (el as HTMLElement));
      setOpen(true);
    };

    el.addEventListener("click", onClick);

    onCleanup(() => {
      el.removeEventListener("click", onClick);
    });
  };

  return { open, isDisabled, attachListeners };
}

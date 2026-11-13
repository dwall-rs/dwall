import { onCleanup, type ValidComponent } from "solid-js";
import { useDialogContext } from "../Dialog";
import type { DialogTriggerProps } from "./DialogTrigger.types";

export interface UseDialogTriggerResult {
  open: () => boolean;
  isDisabled: () => boolean;

  attachListeners: (el: Element) => void;
}

export function useDialogTrigger<T extends ValidComponent>(
  props: () => DialogTriggerProps<T>,
): UseDialogTriggerResult {
  const { open, setOpen } = useDialogContext();

  const isDisabled = () => !!props().disabled;

  const attachListeners = (el: Element) => {
    const onClick = () => {
      if (isDisabled()) return;
      if (open()) return;
      setOpen(true);
    };

    el.addEventListener("click", onClick);

    onCleanup(() => {
      el.removeEventListener("click", onClick);
    });
  };

  return { open, isDisabled, attachListeners };
}

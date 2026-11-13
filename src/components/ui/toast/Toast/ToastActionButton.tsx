import type { JSX } from "solid-js";
import { clsx } from "~/utils";
import type { Action } from "./Toast.types";

export interface ToastActionButtonProps {
  action: Action;

  variant: "action" | "cancel";
  class?: string;
  onClose: () => void;
}

const VARIANT_CLASS: Record<ToastActionButtonProps["variant"], string> = {
  action: "bg-primary text-primary-foreground hover:bg-primary/80",
  cancel: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
};

export function ToastActionButton(props: ToastActionButtonProps): JSX.Element {
  const isAction = () => props.variant === "action";

  return (
    <button
      type="button"
      data-slot={isAction() ? "toast-action" : "toast-cancel"}
      class={clsx(
        "h-6 shrink-0 rounded-[min(var(--radius-md),10px)] px-2 text-xs font-medium outline-hidden transition-colors focus-visible:ring-2 focus-visible:ring-ring/50",
        VARIANT_CLASS[props.variant],
        props.class,
      )}
      onClick={(event) => {
        props.action.onClick?.(event);
        if (!event.defaultPrevented) props.onClose();
      }}
    >
      {props.action.label}
    </button>
  );
}

import { clsx } from "~/utils";
import { X } from "lucide-solid";
import type { ToastT } from "./Toast.types";

export function ToastCloseButton(props: {
  ariaLabel?: string;
  class?: string;
  onClose: () => void;
}) {
  return (
    <button
      type="button"
      data-slot="toast-close"
      aria-label={props.ariaLabel ?? "Close toast"}
      class={clsx(
        "absolute top-1.5 right-1.5 flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring/50",
        props.class,
      )}
      onClick={(): void => props.onClose()}
    >
      <X class="size-3.5" />
    </button>
  );
}

export function isCloseButtonVisible(
  toast: ToastT,
  closeButton: boolean,
): boolean {
  const type = toast.type ?? "default";
  return closeButton && toast.dismissible !== false && type !== "loading";
}

import { Show, type JSX } from "solid-js";
import {
  CircleCheck,
  CircleX,
  Info,
  LoaderCircle,
  TriangleAlert,
} from "lucide-solid";
import { clsx } from "~/utils";
import { toastIconVariants } from "./Toast.styles";
import type { ToastIcons, ToastT, ToastTypes } from "./Toast.types";

export function getDefaultIcon(type: ToastTypes): JSX.Element {
  switch (type) {
    case "success":
      return <CircleCheck />;
    case "info":
      return <Info />;
    case "warning":
      return <TriangleAlert />;
    case "error":
      return <CircleX />;
    case "loading":
      return <LoaderCircle class="animate-spin" />;
    default:
      return undefined;
  }
}

export function resolveToastIcon(
  toast: ToastT,
  icons?: ToastIcons,
): JSX.Element {
  const type = toast.type ?? "default";
  if (toast.icon !== undefined) return toast.icon;
  return icons?.[type as keyof ToastIcons] ?? getDefaultIcon(type);
}

export function ToastIcon(props: { toast: ToastT; icons?: ToastIcons }) {
  return (
    <Show when={resolveToastIcon(props.toast, props.icons)}>
      <div
        data-slot="toast-icon"
        class={clsx(
          toastIconVariants({ type: props.toast.type ?? "default" }),
          props.toast.classes?.icon,
        )}
      >
        {resolveToastIcon(props.toast, props.icons)}
      </div>
    </Show>
  );
}

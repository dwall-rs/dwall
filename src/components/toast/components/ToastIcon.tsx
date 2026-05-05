import { Show, type Component } from "solid-js";
import { Loader, getAsset } from "./Assets";
import type { ToastT, ToastIcons, ToastClassnames } from "../types";
import { clsx } from "~/utils";

interface ToastIconProps {
  toast: ToastT;
  icons?: ToastIcons;
  classes?: ToastClassnames;
}

export const ToastIcon: Component<ToastIconProps> = (props) => {
  const toastType = () => props.toast.type || "default";

  const icon = () =>
    props.toast.icon ??
    props.icons?.[toastType()! as keyof ToastIcons] ??
    getAsset(toastType()!);

  const isLoading = () => toastType() === "loading" && !props.toast.icon;

  const showIconArea = () =>
    (toastType() || props.toast.icon || props.toast.promise) &&
    props.toast.icon !== null &&
    (props.icons?.[toastType()! as keyof ToastIcons] !== null ||
      props.toast.icon);

  return (
    <Show when={showIconArea() && icon()}>
      <div
        data-icon=""
        class={clsx(props.classes?.icon, props.toast.classes?.icon)}
      >
        {/* Loading state: show spinner or custom loading icon */}
        <Show when={props.toast.promise || isLoading()}>
          <Show
            when={props.icons?.loading}
            fallback={
              <Loader
                class={clsx(
                  props.classes?.loader,
                  props.toast.classes?.loader,
                  "sonner-loader",
                )}
                visible={toastType() === "loading"}
              />
            }
          >
            <div
              class={clsx(
                props.classes?.loader,
                props.toast.classes?.loader,
                "sonner-loader",
              )}
              data-visible={toastType() === "loading"}
            >
              {props.icons!.loading}
            </div>
          </Show>
        </Show>

        {/* Non-loading icon */}
        <Show when={toastType() !== "loading"}>{icon()}</Show>
      </div>
    </Show>
  );
};

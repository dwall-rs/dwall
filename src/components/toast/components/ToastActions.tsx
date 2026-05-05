import { type JSX, Show, type Component } from "solid-js";
import { isAction } from "../types";
import type { ToastT, ToastClassnames, Action } from "../types";
import { clsx } from "~/utils";

interface ToastActionsProps {
  toast: ToastT;
  classes?: ToastClassnames;
  cancelButtonStyle?: JSX.CSSProperties;
  actionButtonStyle?: JSX.CSSProperties;
  dismissible: boolean;
  deleteToast: () => void;
}

export const ToastActions: Component<ToastActionsProps> = (props) => {
  return (
    <>
      {/* Cancel button */}
      <Show when={isAction(props.toast.cancel)}>
        <button
          type="button"
          data-button
          data-cancel
          style={props.toast.cancelButtonStyle ?? props.cancelButtonStyle}
          class={clsx(
            props.classes?.cancelButton,
            props.toast.classes?.cancelButton,
          )}
          onClick={(e) => {
            if (!isAction(props.toast.cancel)) return;
            if (!props.dismissible) return;
            props.toast.cancel.onClick?.(e);
            props.deleteToast();
          }}
        >
          {(props.toast.cancel as Action).label}
        </button>
      </Show>

      {/* Cancel as custom element */}
      <Show when={props.toast.cancel && !isAction(props.toast.cancel)}>
        {props.toast.cancel as JSX.Element}
      </Show>

      {/* Action button */}
      <Show when={isAction(props.toast.action)}>
        <button
          type="button"
          data-button
          data-action
          style={props.toast.actionButtonStyle ?? props.actionButtonStyle}
          class={clsx(
            props.classes?.actionButton,
            props.toast.classes?.actionButton,
          )}
          onClick={(e) => {
            if (!isAction(props.toast.action)) return;
            props.toast.action.onClick?.(e);
            if (e.defaultPrevented) return;
            props.deleteToast();
          }}
        >
          {(props.toast.action as Action).label}
        </button>
      </Show>

      {/* Action as custom element */}
      <Show when={props.toast.action && !isAction(props.toast.action)}>
        {props.toast.action as JSX.Element}
      </Show>
    </>
  );
};

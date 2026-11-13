import { Show, type JSX } from "solid-js";
import { isAction, type ToastT } from "./Toast.types";
import { getToastAction } from "./Toast.utils";
import { ToastActionButton } from "./ToastActionButton";

export function ToastActions(props: { toast: ToastT; onClose: () => void }) {
  const action = () => props.toast.action;
  const cancel = () => props.toast.cancel;

  return (
    <>
      <Show when={getToastAction(action())}>
        {(entry) => (
          <ToastActionButton
            action={entry()}
            variant="action"
            class={props.toast.classes?.actionButton}
            onClose={props.onClose}
          />
        )}
      </Show>

      <Show when={getToastAction(cancel())}>
        {(entry) => (
          <ToastActionButton
            action={entry()}
            variant="cancel"
            class={props.toast.classes?.cancelButton}
            onClose={props.onClose}
          />
        )}
      </Show>

      <Show when={action() && !isAction(action())}>
        {action() as JSX.Element}
      </Show>
      <Show when={cancel() && !isAction(cancel())}>
        {cancel() as JSX.Element}
      </Show>
    </>
  );
}

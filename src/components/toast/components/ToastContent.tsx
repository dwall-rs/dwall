import { Show, type Component } from "solid-js";
import type { ToastT, ToastClassnames } from "../types";
import { clsx } from "~/utils";

interface ToastContentProps {
  toast: ToastT;
  classes?: ToastClassnames;
  descriptionClassName?: string;
}

export const ToastContent: Component<ToastContentProps> = (props) => {
  const title = () => {
    if (props.toast.jsx) return props.toast.jsx;
    const t = props.toast.title;
    return typeof t === "function" ? t() : t;
  };

  const description = () => {
    const d = props.toast.description;
    return typeof d === "function" ? d() : d;
  };

  return (
    <div
      data-content=""
      class={clsx(props.classes?.content, props.toast.classes?.content)}
    >
      <div
        data-title=""
        class={clsx(props.classes?.title, props.toast.classes?.title)}
      >
        {title()}
      </div>

      <Show when={props.toast.description}>
        <div
          data-description=""
          class={clsx(
            props.descriptionClassName,
            props.toast.descriptionClass,
            props.classes?.description,
            props.toast.classes?.description,
          )}
        >
          {description()}
        </div>
      </Show>
    </div>
  );
};

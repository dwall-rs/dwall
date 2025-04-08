import {
  createSignal,
  createEffect,
  onCleanup,
  Show,
  type Component,
  onMount,
  mergeProps,
} from "solid-js";
import { useTimeout } from "fluent-solid";
import type { ToastProps } from "./types";
import ToastContainer from "./ToastContainer";
import ToastContent from "./ToastContent";

/**
 * Toast component - Handles Toast lifecycle and state management
 */
const Toast: Component<ToastProps> = (props) => {
  const merged = mergeProps(
    {
      duration: 3000,
      position: "top" as NonNullable<ToastProps["position"]>,
      type: "info" as NonNullable<ToastProps["type"]>,
      closable: true,
      maxWidth: 400,
    },
    props,
  );

  const [visible, setVisible] = createSignal(true);

  const [setTimer, cancelTimer] = useTimeout();

  // Close Toast
  const close = () => {
    setVisible(false);
    props.onClose?.();
  };

  onMount(() => {
    // If ref is provided, pass component instance to ref
    if (props.ref && typeof props.ref === "function") {
      props.ref = (el) => {
        el.close = close;
      };
    }
  });

  // Auto-close timer
  createEffect(() => {
    if (visible() && merged.duration > 0) {
      setTimer(() => {
        close();
      }, merged.duration);
    }
    return () => {
      cancelTimer();
    };
  });

  // Clean up timer when component unmounts
  onCleanup(() => {
    cancelTimer();
  });

  return (
    <Show when={visible()}>
      <ToastContainer
        position={merged.position}
        zIndex={merged.zIndex}
        maxWidth={merged.maxWidth}
        style={props.style}
      >
        <ToastContent
          type={merged.type}
          message={props.message}
          icon={merged.icon}
          action={merged.action}
          closable={merged.closable}
          onClose={close}
        />
      </ToastContainer>
    </Show>
  );
};

export default Toast;

import {
  createSignal,
  createEffect,
  onCleanup,
  Show,
  type Component,
  onMount,
  mergeProps,
  on,
} from "solid-js";
import { usePausableTimeout } from "~/hooks/usePausableTimeout";
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

  // Close Toast
  const close = () => {
    setVisible(false);
    merged.onClose?.();
  };

  const {
    start: startTimer,
    pause: pauseTimer,
    resume: resumeTimer,
    clear: clearTimer,
  } = usePausableTimeout(close, merged.duration);

  onMount(() => {
    // If ref is provided, pass component instance to ref
    if (merged.ref && typeof merged.ref === "function") {
      merged.ref = (el) => {
        el.close = close;
        el.pause = pauseTimer;
        el.resume = () => resumeTimer();
      };
    }
  });

  // Auto-close timer
  createEffect(
    on([visible, () => merged.duration], ([v, duration]) => {
      if (v && duration > 0) {
        startTimer();
      }
      return () => {
        clearTimer();
      };
    }),
  );

  // Clean up timer when component unmounts
  onCleanup(() => {
    clearTimer();
  });

  return (
    <Show when={visible()}>
      <ToastContainer
        position={merged.position}
        zIndex={merged.zIndex}
        maxWidth={merged.maxWidth}
        style={merged.style}
        onMouseEnter={pauseTimer}
        onMouseLeave={() => resumeTimer()}
      >
        <ToastContent
          type={merged.type}
          message={merged.message}
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

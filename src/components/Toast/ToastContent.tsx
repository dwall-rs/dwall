import {
  createEffect,
  createSignal,
  createUniqueId,
  mergeProps,
  on,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import type { ToastContentProps, ToastPosition, ToastType } from "./types";
import ToastIcon from "./ToastIcon";
import { LazyButton } from "~/lazy";
import { VsClose } from "solid-icons/vs";
import { usePausableTimeout } from "~/hooks/usePausableTimeout";
import { useToast } from "./ToastContext";

/**
 * Toast content component - Renders toast content including icon, message and action buttons
 */
const ToastContent: Component<ToastContentProps> = (props) => {
  const { removeToast } = useToast();

  const merged = mergeProps(
    {
      id: createUniqueId(),
      duration: 3000,
      position: "top" as ToastPosition,
      type: "info" as ToastType,
      closable: true,
      maxWidth: 400,
    },
    props,
  );

  const [visible, setVisible] = createSignal(true);
  const [exiting, setExiting] = createSignal(false);

  // Close Toast with animation
  const close = () => {
    setExiting(true);
    // Wait for animation to complete before removing
    const id = setTimeout(() => {
      setVisible(false);
      merged.onClose?.();
      clearTimer();
      removeToast(merged.position, merged.id);
      clearTimeout(id);
    }, 300); // Matches CSS animation duration
  };

  const {
    start: startTimer,
    pause: pauseTimer,
    resume: resumeTimer,
    clear: clearTimer,
  } = usePausableTimeout(close, merged.duration);

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

  const handleMouseEnter = () => {
    pauseTimer();
    merged.onMouseEnter?.();
  };

  const handleMouseLeave = () => {
    resumeTimer();
    merged.onMouseLeave?.();
  };

  return (
    <Show when={visible()}>
      <div
        class={`fluent-toast fluent-toast-${merged.position} ${exiting() ? "fluent-toast-exiting" : ""}`}
        style={{
          ...merged.style,
          "z-index": merged.zIndex,
          "--fui-toast-max-width":
            typeof merged.maxWidth === "number"
              ? `${merged.maxWidth}px`
              : merged.maxWidth,
        }}
        role="alert"
        aria-live="assertive"
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <div class={`fluent-toast-content fluent-toast-${merged.type}`}>
          <div class="fluent-toast-icon-message">
            {merged.icon || <ToastIcon type={merged.type} />}
            <span class="fluent-toast-message">{merged.message}</span>
          </div>
          <div class="fluent-toast-actions">
            {merged.action}
            {merged.closable && (
              <LazyButton
                onClick={close}
                appearance="subtle"
                size="small"
                icon={<VsClose />}
              />
            )}
          </div>
        </div>
      </div>
    </Show>
  );
};

export default ToastContent;

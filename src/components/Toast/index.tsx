import {
  createSignal,
  createEffect,
  onCleanup,
  Show,
  type Component,
  type JSX,
  onMount,
  mergeProps,
} from "solid-js";
import { Portal, render } from "solid-js/web";
import {
  AiFillCheckCircle,
  AiFillCloseCircle,
  AiFillInfoCircle,
  AiFillWarning,
} from "solid-icons/ai";
import { useTimeout } from "fluent-solid";
import "./index.scss";

// Toast type definition
export type ToastType = "info" | "success" | "warning" | "error";

export type ToastPosition =
  | "top"
  | "bottom"
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right";

type ToastRef = HTMLDivElement & {
  close: () => void;
};

export interface ToastProps
  extends Omit<JSX.HTMLAttributes<HTMLDivElement>, "ref" | "style"> {
  /** Reference to the Toast component instance */
  ref?: ToastRef | ((el: ToastRef) => void);
  /** Content to display in the toast */
  message: JSX.Element;
  /** Duration in milliseconds before the toast auto-closes. Default: 3000ms */
  duration?: number;
  /** Position where the toast appears */
  position?: ToastPosition;
  /** Type of the toast notification */
  type?: ToastType;
  /** Whether the toast can be manually closed */
  closable?: boolean;
  /** Callback function triggered when toast closes */
  onClose?: () => void;
  /** Custom icon element to display */
  icon?: JSX.Element;
  /** Custom action button element */
  action?: JSX.Element;
  /** Maximum number of toasts to show simultaneously */
  maxCount?: number;
  /** Custom z-index value */
  zIndex?: number;
  /** Custom CSS styles */
  style?: JSX.CSSProperties;
}

const Toast: Component<ToastProps> = (props) => {
  const merged = mergeProps(
    {
      duration: 3000,
      position: "top",
      type: "info" as ToastType,
      closable: true,
    },
    props,
  );

  const [visible, setVisible] = createSignal(true);

  const [setTimer, cancelTimer] = useTimeout();

  // Close the toast
  const close = () => {
    setVisible(false);
    props.onClose?.();
  };

  onMount(() => {
    // If ref is provided, pass the component instance to ref
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

  // Clear timer on component unmount
  onCleanup(() => {
    cancelTimer();
  });

  return (
    <Show when={visible()}>
      <Portal
        ref={(el) => {
          el.classList.add("fluent-toast-container");
        }}
      >
        <div
          class={`fluent-toast fluent-toast-${merged.position}`}
          style={{
            ...props.style,
            "z-index": merged.zIndex,
          }}
          role="alert"
          aria-live="assertive"
        >
          <div class={`fluent-toast-content fluent-toast-${merged.type}`}>
            <div class="fluent-toast-icon-message">
              {merged.icon || <ToastIcon type={merged.type} />}
              <span class="fluent-toast-message">{props.message}</span>
            </div>
            <div class="fluent-toast-actions">
              {merged.action}
              {merged.closable && (
                <button
                  type="button"
                  class="fluent-toast-close"
                  onClick={close}
                  aria-label="Close notification"
                >
                  ×
                </button>
              )}
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
};

// Create a global Toast manager
let toastContainer: HTMLDivElement | null = null;

// Ensure Toast container exists
function ensureContainer() {
  if (!toastContainer) {
    toastContainer = document.createElement("div");
    toastContainer.className = "fluent-toast-container";
    document.body.appendChild(toastContainer);
  }
  return toastContainer;
}

// Toast icon component
const ToastIcon: Component<{ type: ToastType }> = (props) => {
  const iconMap = {
    info: <AiFillInfoCircle />,
    success: <AiFillCheckCircle />,
    warning: <AiFillWarning />,
    error: <AiFillCloseCircle />,
  };

  return <span class="fluent-toast-icon">{iconMap[props.type]}</span>;
};

// Global Toast queue management
let toastQueue: Array<{ id: string; close: () => void }> = [];
let maxToastCount = 5; // Default maximum display count

// Function to display Toast
export function showToast(options: ToastProps) {
  const container = ensureContainer();

  // Set maximum display count
  if (options.maxCount !== undefined) {
    maxToastCount = options.maxCount;
  }

  // If exceeding maximum display count, close the oldest Toast
  if (toastQueue.length >= maxToastCount) {
    const oldestToast = toastQueue.shift();
    oldestToast?.close();
  }

  // Convert to object if input is a string
  const props: ToastProps = options;

  // Create a new div as Toast container
  const toastElement = document.createElement("div");
  container.appendChild(toastElement);

  // Generate unique ID
  const toastId = `toast-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  // Create a function to remove toast element
  const removeToast = () => {
    if (container.contains(toastElement)) {
      container.removeChild(toastElement);
    }

    // Remove container if empty
    if (container.childNodes.length === 0) {
      document.body.removeChild(container);
      toastContainer = null;
    }
  };

  // Extend props with onClose callback
  const toastProps: ToastProps = {
    ...props,
    onClose: () => {
      props.onClose?.();
      setTimeout(removeToast, 300); // Remove element after animation ends
    },
  };

  // Create a controllable close function
  let closeToast: () => void;

  render(
    () => (
      <Toast
        {...toastProps}
        ref={(el) => {
          // Get the close method from component instance
          closeToast = () => {
            el?.close();
          };
        }}
      />
    ),
    toastElement,
  );

  // Create toast reference object
  const toastRef = {
    id: toastId,
    close: () => {
      // Trigger toast close logic
      closeToast?.();
      // Remove from queue
      toastQueue = toastQueue.filter((t) => t.id !== toastId);
    },
  };

  // Add to queue
  toastQueue.push(toastRef);

  // Return toast reference
  return toastRef;
}

export const toast = {
  info: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => {
    return showToast({ message, type: "info", ...options });
  },
  success: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => {
    return showToast({ message, type: "success", ...options });
  },
  warning: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => {
    return showToast({ message, type: "warning", ...options });
  },
  error: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => {
    return showToast({ message, type: "error", ...options });
  },
  closeAll: () => {
    for (const toast of toastQueue) {
      toast.close();
    }
    toastQueue = [];
  },
};

export default Toast;

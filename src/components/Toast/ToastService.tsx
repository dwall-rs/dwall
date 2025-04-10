import { render } from "solid-js/web";
import type { JSX } from "solid-js";
import type { ToastInstance, ToastProps, ToastServiceInterface } from "./types";
import Toast from "./Toast";

// Global Toast container
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

// Global Toast queue management
let toastQueue: Array<ToastInstance> = [];
let maxToastCount = 5; // Default maximum display count

/**
 * Function to show Toast
 */
export function showToast(options: ToastProps): ToastInstance {
  const container = ensureContainer();

  // Set maximum display count
  if (options.maxCount !== undefined) {
    maxToastCount = options.maxCount;
  }

  // If exceeds maximum display count, close the oldest Toast
  if (toastQueue.length >= maxToastCount) {
    const oldestToast = toastQueue.shift();
    oldestToast?.close();
  }

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

    // If container is empty, remove it
    if (container.childNodes.length === 0) {
      document.body.removeChild(container);
      toastContainer = null;
    }
  };

  // Extend props, add onClose callback
  const toastProps: ToastProps = {
    ...options,
    onClose: () => {
      options.onClose?.();
      setTimeout(removeToast, 300); // Remove element after animation
    },
  };

  // Create a controllable close function
  let closeToast: () => void;

  render(
    () => (
      <Toast
        {...toastProps}
        ref={(el) => {
          // Get close method from component instance
          closeToast = () => {
            el?.close();
          };
        }}
      />
    ),
    toastElement,
  );

  // Create toast reference object
  const toastRef: ToastInstance = {
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

/**
 * Toast service - Provides methods to create different types of Toasts
 */
export const toast: ToastServiceInterface = {
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

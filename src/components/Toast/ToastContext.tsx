import {
  createContext,
  createUniqueId,
  For,
  type JSX,
  Show,
  useContext,
} from "solid-js";
import { createStore } from "solid-js/store";
import type { ToastContentProps, ToastPosition } from "./types";
import ToastContainer from "./ToastContainer";
import ToastContent from "./ToastContent";
import "./index.scss";

type ToastOptions = Partial<Omit<ToastContentProps, "message" | "type">>;
type ToastFunction = (message: JSX.Element, options?: ToastOptions) => string;

interface ToastContext {
  addToast: (toast: ToastContentProps) => void;
  removeToast: (position: ToastPosition, id: string) => void;
  updateToast: (
    position: ToastPosition,
    id: string,
    toast: Partial<ToastContentProps>,
  ) => void;
  success: ToastFunction;
  error: ToastFunction;
  warning: ToastFunction;
  info: ToastFunction;
}

const ToastContext = createContext<ToastContext>();

export const ToastProvider = (props: { children: JSX.Element }) => {
  const MAX_TOASTS_PER_POSITION = 5;
  const [toasts, setToasts] = createStore<
    Record<ToastPosition, ToastContentProps[]>
  >({
    top: [],
    bottom: [],
    "top-left": [],
    "top-right": [],
    "bottom-left": [],
    "bottom-right": [],
  });

  // Add toast, remove the oldest one if exceeds max count
  const addToast = (toast: ToastContentProps) => {
    toast.id = toast.id ?? createUniqueId();
    const position = toast.position ?? "top";
    const isTopPosition = position.startsWith("top");

    setToasts(position, (currentToasts) => {
      let newToasts = [...currentToasts];

      // Remove the oldest one if exceeds max count
      if (currentToasts.length >= MAX_TOASTS_PER_POSITION) {
        // Remove the oldest toast (for top positions remove the first one,
        // for bottom positions remove the last one)
        newToasts = isTopPosition ? newToasts.slice(1) : newToasts.slice(0, -1);
      }

      // Add new toast based on position (for top positions add to the end,
      // for bottom positions add to the beginning)
      return isTopPosition ? [...newToasts, toast] : [toast, ...newToasts];
    });

    return toast.id;
  };

  // Remove specified toast
  const removeToast = (position: ToastPosition, id: string) => {
    setToasts(position, (toasts) => toasts.filter((toast) => toast.id !== id));
  };

  // Update specified toast
  const updateToast = (
    position: ToastPosition,
    id: string,
    updatedToast: Partial<ToastContentProps>,
  ) => {
    setToasts(position, (toasts) =>
      toasts.map((toast) =>
        toast.id === id ? { ...toast, ...updatedToast } : toast,
      ),
    );
  };

  // Helper method: show success toast
  const success = (message: JSX.Element, options?: ToastOptions) => {
    return addToast({
      type: "success",
      message,
      ...options,
    } as ToastContentProps);
  };

  // Helper method: show error toast
  const error = (message: JSX.Element, options?: ToastOptions) => {
    return addToast({
      type: "error",
      message,
      ...options,
    } as ToastContentProps);
  };

  // Helper method: show warning toast
  const warning = (message: JSX.Element, options?: ToastOptions) => {
    return addToast({
      type: "warning",
      message,
      ...options,
    } as ToastContentProps);
  };

  // Helper method: show info toast
  const info = (message: JSX.Element, options?: ToastOptions) => {
    return addToast({
      type: "info",
      message,
      ...options,
    } as ToastContentProps);
  };

  // Render toast container for specified position
  const renderToastContainer = (position: ToastPosition) => {
    return (
      <Show when={toasts[position].length > 0}>
        <ToastContainer position={position}>
          <For each={toasts[position]}>
            {(toast) => <ToastContent {...toast} />}
          </For>
        </ToastContainer>
      </Show>
    );
  };

  // All possible positions
  const positions: ToastPosition[] = [
    "top",
    "bottom",
    "top-left",
    "top-right",
    "bottom-left",
    "bottom-right",
  ];

  return (
    <ToastContext.Provider
      value={{
        addToast,
        removeToast,
        updateToast,
        success,
        error,
        warning,
        info,
      }}
    >
      {props.children}

      {positions.map((position) => renderToastContainer(position))}
    </ToastContext.Provider>
  );
};

export const useToast = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast: cannot find a ToastContext");
  }
  return context;
};

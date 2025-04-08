import type { JSX } from "solid-js";

export type ToastType = "info" | "success" | "warning" | "error";

export type ToastPosition =
  | "top"
  | "bottom"
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right";

export type ToastRef = HTMLDivElement & {
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

export interface ToastIconProps {
  type: ToastType;
}

export interface ToastContentProps {
  type: ToastType;
  message: JSX.Element;
  icon?: JSX.Element;
  action?: JSX.Element;
  closable?: boolean;
  onClose: () => void;
}

export interface ToastContainerProps {
  position: ToastPosition;
  zIndex?: number;
  style?: JSX.CSSProperties;
  children: JSX.Element;
}

export interface ToastInstance {
  id: string;
  close: () => void;
}

export interface ToastServiceInterface {
  info: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => ToastInstance;
  success: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => ToastInstance;
  warning: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => ToastInstance;
  error: (
    message: JSX.Element,
    options?: Omit<ToastProps, "message" | "type">,
  ) => ToastInstance;
  closeAll: () => void;
}

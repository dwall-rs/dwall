import type { JSX } from "solid-js";

export type ToastType = "info" | "success" | "warning" | "error";

export type ToastPosition =
  | "top"
  | "bottom"
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right";

export interface ToastIconProps {
  type: ToastType;
}

export interface ToastContentProps {
  id?: string;
  type?: ToastType;
  message: JSX.Element;
  icon?: JSX.Element;
  action?: JSX.Element;
  closable?: boolean;
  onClose?: () => void;
  position?: ToastPosition;
  zIndex?: number;
  maxWidth?: number | string;
  style?: JSX.CSSProperties;
  onMouseEnter?: () => void;
  onMouseLeave?: () => void;
  duration?: number;
}

export interface ToastContainerProps {
  children: JSX.Element;
  position: ToastPosition;
}

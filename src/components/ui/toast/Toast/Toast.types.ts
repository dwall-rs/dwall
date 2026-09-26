import type { JSX, JSXElement } from "solid-js";

export type ToastTypes =
  | "success"
  | "info"
  | "warning"
  | "error"
  | "loading"
  | "default";

export type Position =
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right"
  | "top-center"
  | "bottom-center";

export type Theme = "light" | "dark";
export type SwipeDirection = "top" | "right" | "bottom" | "left";

export interface Action {
  label: JSXElement;
  onClick: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
  actionButtonStyle?: JSX.CSSProperties;
}

export interface ToastIcons {
  success?: JSXElement;
  info?: JSXElement;
  warning?: JSXElement;
  error?: JSXElement;
  loading?: JSXElement;
  close?: JSXElement;
}

export interface ToastClassnames {
  toast?: string;
  title?: string;
  description?: string;
  loader?: string;
  cancelButton?: string;
  actionButton?: string;
  closeButton?: string;
  success?: string;
  error?: string;
  info?: string;
  warning?: string;
  loading?: string;
  default?: string;
  content?: string;
  icon?: string;
}

export type PromiseT<Data = any> = Promise<Data> | (() => Promise<Data>);

export type PromiseTResult<Data = any> =
  | string
  | JSXElement
  | ((data: Data) => JSXElement | string | Promise<JSXElement | string>);

export interface PromiseData<ToastData = any> {
  loading?: string | JSXElement;
  success?: PromiseTResult<ToastData>;
  error?: PromiseTResult;
  description?: PromiseTResult;
  finally?: () => void | Promise<void>;
}

export interface ToastT {
  id: string;
  toasterId?: string;
  title?: JSXElement | (() => JSXElement);
  description?: JSXElement | (() => JSXElement);
  type?: ToastTypes;
  icon?: JSXElement;
  jsx?: JSXElement;
  richColors?: boolean;
  invert?: boolean;
  closeButton?: boolean;
  dismissible?: boolean;
  duration?: number;
  delete?: boolean;
  action?: Action | JSXElement;
  cancel?: Action | JSXElement;
  onDismiss?: (toast: ToastT) => void;
  onAutoClose?: (toast: ToastT) => void;
  promise?: PromiseT;
  style?: JSX.CSSProperties;
  class?: string;
  classes?: ToastClassnames;
  descriptionClass?: string;
  position?: Position;
  testId?: string;
}

export type ExternalToast = Omit<
  ToastT,
  "id" | "type" | "title" | "jsx" | "delete" | "promise"
> & {
  id?: string;
  toasterId?: string;
};

export interface ToastProps {
  toast: ToastT;
  closeButton: boolean;
  duration?: number;
  class?: string;
  icons?: ToastIcons;
  closeButtonAriaLabel?: string;
  defaultRichColors?: boolean;
  onRemove: (id: string) => void;
  index: number;
  total: number;
  expanded: boolean;
  position: Position;
  gap: number;
}

export const isAction = (action: Action | JSXElement): action is Action => {
  return (action as Action)?.label !== undefined;
};

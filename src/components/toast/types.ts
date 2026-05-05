import type { JSX, JSXElement, Setter } from "solid-js";

export type ToastTypes =
  // | "normal"
  // | "action"
  "success" | "info" | "warning" | "error" | "loading" | "default";

export type PromiseT<Data = any> = Promise<Data> | (() => Promise<Data>);

export interface PromiseIExtendedResult extends ExternalToast {
  message: JSXElement;
}

export type PromiseTExtendedResult<Data = any> =
  | PromiseIExtendedResult
  | ((data: Data) => PromiseIExtendedResult | Promise<PromiseIExtendedResult>);

export type PromiseTResult<Data = any> =
  | string
  | JSXElement
  | ((data: Data) => JSXElement | string | Promise<JSXElement | string>);

export type PromiseExternalToast = Omit<ExternalToast, "description">;

export type PromiseData<ToastData = any> = PromiseExternalToast & {
  loading?: string | JSXElement;
  success?: PromiseTResult<ToastData> | PromiseTExtendedResult<ToastData>;
  error?: PromiseTResult | PromiseTExtendedResult;
  description?: PromiseTResult;
  finally?: () => void | Promise<void>;
};

export interface ToastClassnames {
  toast?: string;
  title?: string;
  description?: string;
  loader?: string;
  cancelButton?: string;
  actionButton?: string;
  success?: string;
  error?: string;
  info?: string;
  warning?: string;
  loading?: string;
  default?: string;
  content?: string;
  icon?: string;
}

export interface ToastIcons {
  success?: JSXElement;
  info?: JSXElement;
  warning?: JSXElement;
  error?: JSXElement;
  loading?: JSXElement;
  close?: JSXElement;
}

export interface Action {
  label: JSXElement;
  onClick: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
  actionButtonStyle?: JSX.CSSProperties;
}

export interface ToastT {
  id: string;
  toasterId?: string;
  title?: (() => JSXElement) | JSXElement;
  type?: ToastTypes;
  icon?: JSXElement;
  jsx?: JSXElement;
  richColors?: boolean;
  invert?: boolean;
  closeButton?: boolean;
  dismissible?: boolean;
  description?: (() => JSXElement) | JSXElement;
  duration?: number;
  delete?: boolean;
  action?: Action | JSXElement;
  cancel?: Action | JSXElement;
  onDismiss?: (toast: ToastT) => void;
  onAutoClose?: (toast: ToastT) => void;
  promise?: PromiseT;
  cancelButtonStyle?: JSX.CSSProperties;
  actionButtonStyle?: JSX.CSSProperties;
  style?: JSX.CSSProperties;
  unstyled?: boolean;
  class?: string;
  classes?: ToastClassnames;
  descriptionClass?: string;
  position?: Position;
  testId?: string;
}

export const isAction = (action: Action | JSXElement): action is Action => {
  return (action as Action)?.label !== undefined;
};

export type Position =
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right"
  | "top-center"
  | "bottom-center";
export interface HeightT {
  height: number;
  toastId: number | string;
  position: Position;
}

interface ToastOptions {
  class?: string;
  closeButton?: boolean;
  descriptionClass?: string;
  style?: JSX.CSSProperties;
  cancelButtonStyle?: JSX.CSSProperties;
  actionButtonStyle?: JSX.CSSProperties;
  duration?: number;
  unstyled?: boolean;
  classes?: ToastClassnames;
  closeButtonAriaLabel?: string;
  toasterId?: string;
}

type Offset =
  | {
      top?: string | number;
      right?: string | number;
      bottom?: string | number;
      left?: string | number;
    }
  | string
  | number;

export interface ToasterProps {
  id?: string;
  invert?: boolean;
  theme?: "light" | "dark" | "system";
  position?: Position;
  hotkey?: string[];
  richColors?: boolean;
  expand?: boolean;
  duration?: number;
  gap?: number;
  visibleToasts?: number;
  closeButton?: boolean;
  toastOptions?: ToastOptions;
  class?: string;
  style?: JSX.CSSProperties;
  offset?: Offset;
  mobileOffset?: Offset;
  dir?: "rtl" | "ltr" | "auto";
  swipeDirections?: SwipeDirection[];
  icons?: ToastIcons;
  customAriaLabel?: string;
  containerAriaLabel?: string;
}

export type SwipeDirection = "top" | "right" | "bottom" | "left";

export interface ToastProps {
  toast: ToastT;
  toasts: ToastT[];
  index: number;
  swipeDirections?: SwipeDirection[];
  expanded: boolean;
  invert: boolean;
  heights: HeightT[];
  setHeights: Setter<HeightT[]>;
  removeToast: (toast: ToastT) => void;
  gap?: number;
  position: Position;
  visibleToasts: number;
  expandByDefault: boolean;
  closeButton: boolean;
  interacting: boolean;
  style?: JSX.CSSProperties;
  cancelButtonStyle?: JSX.CSSProperties;
  actionButtonStyle?: JSX.CSSProperties;
  duration?: number;
  class?: string;
  unstyled?: boolean;
  descriptionClass?: string;
  loadingIcon?: JSXElement;
  classes?: ToastClassnames;
  icons?: ToastIcons;
  closeButtonAriaLabel?: string;
  defaultRichColors?: boolean;
}

export enum SwipeStateTypes {
  SwipedOut = "SwipedOut",
  SwipedBack = "SwipedBack",
  NotSwiped = "NotSwiped",
}

export type Theme = "light" | "dark";

export interface ToastToDismiss {
  id: string;
  dismiss: boolean;
}

export type ExternalToast = Omit<
  ToastT,
  "id" | "type" | "title" | "jsx" | "delete" | "promise"
> & {
  id?: string;
  toasterId?: string;
};

import { For, type JSX } from "solid-js";
import { clsx } from "~/utils";
import { Toast } from "../Toast";
import type {
  Position,
  ToastIcons,
  ToastT,
  ToastVariant,
} from "../Toast/Toast.types";
import { toasterContainerClass } from "./Toaster.styles";
import type { Offset, ToastOptions } from "./Toaster.types";
import {
  getPositionClass,
  getViewportLayoutClass,
  resolveOffsetStyle,
} from "./Toaster.utils";

export interface ToastViewportProps {
  position: Position;

  expanded: boolean;

  toasts: ToastT[];
  gap: number;
  ariaLabel: string;
  offset?: Offset;
  mobileOffset?: Offset;
  style?: JSX.CSSProperties;

  closeButton?: boolean;
  duration?: number;
  icons?: ToastIcons;

  variant?: ToastVariant;
  richColors?: boolean;
  toastOptions?: ToastOptions;

  onExpandChange: (expanded: boolean) => void;
  onRemove: (id: string) => void;
}

export function ToastViewport(props: ToastViewportProps) {
  return (
    <ol
      data-slot="toaster-viewport"
      data-position={props.position}
      aria-live="polite"
      aria-relevant="additions text"
      aria-atomic="false"
      aria-label={props.ariaLabel}
      tabIndex={-1}
      class={clsx(
        toasterContainerClass,
        getPositionClass(props.position),
        getViewportLayoutClass(props.position, props.expanded),

        "pl-safe pr-safe",
        props.position.startsWith("bottom") ? "pb-safe" : "pt-safe",
      )}
      style={{
        gap: `${props.gap}px`,
        ...resolveOffsetStyle(props.position, props.offset, props.mobileOffset),
        ...(props.style as Record<string, string | number>),
      }}
      onMouseEnter={() => props.onExpandChange(true)}
      onMouseLeave={() => props.onExpandChange(false)}
    >
      <For each={props.toasts}>
        {(toast: ToastT, index) => (
          <Toast
            toast={toast}
            index={index()}
            total={props.toasts.length}
            expanded={props.expanded}
            position={props.position}
            gap={props.gap}
            closeButton={
              toast.closeButton ??
              props.toastOptions?.closeButton ??
              props.closeButton ??
              false
            }
            duration={props.toastOptions?.duration ?? props.duration}
            class={props.toastOptions?.class}
            icons={props.icons}
            closeButtonAriaLabel={props.toastOptions?.closeButtonAriaLabel}
            variant={props.toastOptions?.variant ?? props.variant}
            defaultRichColors={props.richColors}
            onRemove={props.onRemove}
          />
        )}
      </For>
    </ol>
  );
}

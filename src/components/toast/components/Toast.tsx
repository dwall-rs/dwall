/**
 * components/Toast.tsx
 * Renders a single toast item.
 * Composes sub-components (ToastIcon, ToastContent, ToastActions) and hooks
 * (useToastTimer, useSwipe, useToastHeight). No business logic lives here directly.
 */
import {
  createEffect,
  createMemo,
  createSignal,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { ToastIcon } from "./ToastIcon";
import { ToastContent } from "./ToastContent";
import { ToastActions } from "./ToastActions";
import { useIsDocumentHidden } from "../hooks";
import { useToastTimer } from "../hooks/useToastTimer";
import { useSwipe } from "../hooks/useSwipe";
import { useToastHeight } from "../hooks/useToastHeight";
import { TIME_BEFORE_UNMOUNT, TOAST_LIFETIME } from "../constants";
import type { ToastProps } from "../types";
import { clsx } from "~/utils";
import { Button } from "~/components/button";
import { X } from "lucide-solid";

export const Toast: Component<ToastProps> = (props) => {
  let toastRef: HTMLLIElement | undefined;

  const [mounted, setMounted] = createSignal(false);
  const [removed, setRemoved] = createSignal(false);

  const isDocumentHidden = useIsDocumentHidden();

  // Derived values
  const isFront = () => props.index === 0;
  const isVisible = () => props.index + 1 <= props.visibleToasts;
  const dismissible = () => props.toast.dismissible !== false;
  const toastType = () => props.toast.type ?? "default";
  const disabled = () => toastType() === "loading";

  const closeButton = () => props.toast.closeButton ?? props.closeButton;
  const duration = () =>
    props.toast.duration ?? props.duration ?? TOAST_LIFETIME;

  const [y, x] = props.position.split("-");

  const heightIndex = createMemo(
    () => props.heights.findIndex((h) => h.toastId === props.toast.id) || 0,
  );

  const toastsHeightBefore = createMemo(() =>
    props.heights.reduce((acc, curr, idx) => {
      if (idx >= heightIndex()) return acc;
      return acc + curr.height;
    }, 0),
  );

  const offset = createMemo(
    () => heightIndex() * (props.gap ?? 14) + toastsHeightBefore(),
  );

  // ─── Mount animation ─────────────────────────────────────────────────────────
  onMount(() => setMounted(true));

  // ─── Delete helper ───────────────────────────────────────────────────────────
  const deleteToast = () => {
    setRemoved(true);
    swipe.setOffsetBeforeRemove(offset());
    props.setHeights((h) => h.filter((x) => x.toastId !== props.toast.id));
    setTimeout(() => props.removeToast(props.toast), TIME_BEFORE_UNMOUNT);
  };

  // ─── Sub-hooks ───────────────────────────────────────────────────────────────
  const { initialHeight } = useToastHeight({
    toastRef: () => toastRef,
    toast: () => props.toast,
    mounted,
    setHeights: props.setHeights,
  });

  useToastTimer({
    toast: props.toast,
    duration: duration(),
    expanded: () => props.expanded,
    interacting: () => props.interacting,
    isDocumentHidden,
    deleteToast,
  });

  const swipe = useSwipe({
    toastRef: () => toastRef,
    toast: props.toast,
    dismissible: dismissible(),
    position: props.position,
    swipeDirections: props.swipeDirections,
    offset,
    deleteToast,
  });

  // ─── External delete signal ──────────────────────────────────────────────────
  createEffect(() => {
    if (props.toast.delete) {
      deleteToast();
      props.toast.onDismiss?.(props.toast);
    }
  });

  return (
    <li
      // tabIndex={0}
      ref={toastRef}
      class={clsx(
        props.class,
        props.toast.class,
        props.classes?.toast,
        props.toast.classes?.toast,
        props.classes?.default,
        props.classes?.[toastType()],
        props.toast.classes?.[toastType()],
      )}
      data-sonner-toast=""
      data-rich-colors={props.toast.richColors ?? props.defaultRichColors}
      data-styled={!(props.toast.jsx || props.toast.unstyled || props.unstyled)}
      data-mounted={mounted()}
      data-promise={Boolean(props.toast.promise)}
      data-swiped={swipe.isSwiped()}
      data-removed={removed()}
      data-visible={isVisible()}
      data-y-position={y}
      data-x-position={x}
      data-index={props.index}
      data-front={isFront()}
      data-swiping={swipe.swiping()}
      data-dismissible={dismissible()}
      data-type={toastType()}
      data-invert={props.toast.invert || props.invert}
      data-swipe-out={swipe.swipeOut()}
      data-swipe-direction={swipe.swipeOutDirection()}
      data-expanded={props.expanded || (props.expandByDefault && mounted())}
      data-testid={props.toast.testId}
      style={{
        "--index": props.index,
        "--toasts-before": props.index,
        "--z-index": props.toasts.length - props.index,
        "--offset": `${removed() ? swipe.offsetBeforeRemove() : offset()}px`,
        "--initial-height": props.expandByDefault
          ? "auto"
          : `${initialHeight()}px`,
        ...props.style,
        ...props.toast.style,
      }}
      onDragEnd={swipe.onDragEnd}
      onPointerDown={swipe.onPointerDown}
      onPointerUp={swipe.onPointerUp}
      onPointerMove={swipe.onPointerMove}
    >
      {/* Close button */}
      <Show
        when={
          closeButton() && !props.toast.jsx && props.toast.type !== "loading"
        }
      >
        <Button
          aria-label={props.closeButtonAriaLabel ?? "Close toast"}
          data-disabled={disabled()}
          data-close-button
          onClick={
            disabled() || !dismissible()
              ? undefined
              : () => {
                  deleteToast();
                  props.toast.onDismiss?.(props.toast);
                }
          }
          icon={{ icon: <X />, ariaLabel: "Close" }}
          size="xs"
        />
      </Show>

      {/* Icon area */}
      <ToastIcon
        toast={props.toast}
        icons={props.icons}
        classes={props.classes}
      />

      {/* Text content */}
      <ToastContent
        toast={props.toast}
        classes={props.classes}
        descriptionClassName={props.descriptionClass}
      />

      {/* Action buttons */}
      <ToastActions
        toast={props.toast}
        classes={props.classes}
        cancelButtonStyle={props.cancelButtonStyle}
        actionButtonStyle={props.actionButtonStyle}
        dismissible={dismissible()}
        deleteToast={deleteToast}
      />
    </li>
  );
};

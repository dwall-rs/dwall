import { Show, createMemo, splitProps } from "solid-js";
import { clsx } from "~/utils";
import { toastVariants } from "./Toast.styles";
import type {
  ToastIcons,
  ToastProps,
  ToastT,
  ToastVariant,
} from "./Toast.types";
import { getAnimationClasses, getToastStyle } from "./Toast.utils";
import { ToastActions } from "./ToastActions";
import { ToastCloseButton, isCloseButtonVisible } from "./ToastCloseButton";
import { ToastContent } from "./ToastContent";
import { ToastIcon } from "./ToastIcon";
import { useToastLifecycle } from "./useToastLifecycle";

export function Toast(props: ToastProps) {
  const [local] = splitProps(props, [
    "toast",
    "closeButton",
    "duration",
    "class",
    "icons",
    "closeButtonAriaLabel",
    "variant",
    "defaultRichColors",
    "onRemove",
    "index",
    "total",
    "expanded",
    "position",
    "gap",
  ]);

  const toast = () => local.toast;
  const toastType = createMemo(() => toast().type ?? "default");
  const richColors = createMemo(
    () => toast().richColors ?? local.defaultRichColors ?? false,
  );

  const variant = createMemo<ToastVariant>(
    () =>
      toast().variant ??
      local.variant ??
      (richColors() ? "semantic" : "popover"),
  );

  const { animationState, close, hold, release } = useToastLifecycle({
    toast,
    duration: () => local.duration,
    onRemove: (id) => local.onRemove(id),
  });

  let heldByPointer = false;
  let heldByFocus = false;
  const updateHold = () => {
    if (heldByPointer || heldByFocus) {
      hold();
    } else {
      release();
    }
  };

  const toastStyle = () =>
    getToastStyle({
      toast: toast(),
      index: local.index,
      total: local.total,
      expanded: local.expanded,
      position: local.position,
      gap: local.gap,
    });

  const animationClasses = () =>
    getAnimationClasses({
      index: local.index,
      expanded: local.expanded,
      position: local.position,
    });

  return (
    <li
      data-slot="toast"
      onPointerEnter={() => {
        heldByPointer = true;
        updateHold();
      }}
      onPointerLeave={() => {
        heldByPointer = false;
        updateHold();
      }}
      onFocusIn={() => {
        heldByFocus = true;
        updateHold();
      }}
      onFocusOut={(event) => {
        const next = event.relatedTarget;
        const element = event.currentTarget;
        if (next instanceof Node && element.contains(next)) return;

        heldByFocus = false;
        updateHold();
      }}
      data-type={toastType()}
      data-state={animationState()}
      data-variant={variant()}
      data-rich-colors={richColors()}
      data-testid={toast().testId}
      class={clsx(
        toastVariants({
          type: toastType(),
          variant: variant(),
        }),
        local.closeButton && "pr-8",
        animationClasses(),
        local.class,
        toast().class,
        toast().classes?.toast,
        toast().classes?.[toastType()],
      )}
      style={toastStyle()}
    >
      <Show
        when={toast().jsx}
        fallback={
          <ToastBody
            toast={toast()}
            icons={local.icons}
            closeButton={local.closeButton}
            closeButtonAriaLabel={local.closeButtonAriaLabel}
            onClose={close}
          />
        }
      >
        {toast().jsx}
      </Show>
    </li>
  );
}

function ToastBody(props: {
  toast: ToastT;
  icons?: ToastIcons;
  closeButton: boolean;
  closeButtonAriaLabel?: string;
  onClose: () => void;
}) {
  return (
    <>
      <ToastIcon toast={props.toast} icons={props.icons} />
      <ToastContent toast={props.toast} />
      <ToastActions toast={props.toast} onClose={props.onClose} />
      <Show when={isCloseButtonVisible(props.toast, props.closeButton)}>
        <ToastCloseButton
          ariaLabel={props.closeButtonAriaLabel}
          class={props.toast.classes?.closeButton}
          onClose={props.onClose}
        />
      </Show>
    </>
  );
}

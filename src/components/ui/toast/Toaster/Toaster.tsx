import {
  For,
  createMemo,
  createSignal,
  mergeProps,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { Portal } from "solid-js/web";
import { clsx } from "~/utils";
import { Toast } from "../Toast";
import type { Position, ToastT } from "../Toast/Toast.types";
import { removeToast, useSonner } from "../state/toast";
import type { ToasterProps } from "./Toaster.types";
import { toasterContainerClass } from "./Toaster.styles";
import {
  getDocumentDirection,
  getPositionClass,
  resolveOffsetStyle,
} from "./Toaster.utils";

export const Toaster: Component<ToasterProps> = (props) => {
  const merged = mergeProps(
    {
      position: "bottom-right",
      gap: 14,
      visibleToasts: 3,
      dir: getDocumentDirection(),
      containerAriaLabel: "Notifications",
      hotkey: ["altKey", "KeyT"] as string[],
    } as const,
    props,
  );

  const { toasts } = useSonner();
  const [expanded, setExpanded] = createSignal(false);

  const filteredToasts = createMemo(() => {
    const id = merged.id;
    return id
      ? toasts.filter((toast) => toast.toasterId === id)
      : toasts.filter((toast) => !toast.toasterId);
  });

  const possiblePositions = createMemo(() =>
    Array.from(
      new Set(
        [merged.position].concat(
          filteredToasts()
            .filter((toast) => toast.position)
            .map((toast) => toast.position as Position),
        ),
      ),
    ),
  );

  const toastsForPosition = (position: Position) =>
    filteredToasts().filter((toast) => {
      if (toast.position) return toast.position === position;
      return position === merged.position;
    });

  const visibleToastsForPosition = (position: Position) => {
    const list = toastsForPosition(position);
    if (merged.expand) return list;
    return list.slice(0, merged.visibleToasts);
  };

  onMount(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const hotkeyPressed =
        merged.hotkey.length > 0 &&
        merged.hotkey.every((key) => (e as any)[key] || e.code === key);
      if (hotkeyPressed) setExpanded(true);
      if (e.code === "Escape") setExpanded(false);
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  const offsetStyle = (position: Position) =>
    resolveOffsetStyle(position, merged.offset, merged.mobileOffset);

  return (
    <Portal>
      <div
        data-slot="toaster"
        dir={merged.dir === "auto" ? getDocumentDirection() : merged.dir}
      >
        <For each={possiblePositions()}>
          {(position) => (
            <Show when={visibleToastsForPosition(position).length > 0}>
              <ol
                data-slot="toaster-viewport"
                data-position={position}
                aria-live="polite"
                aria-relevant="additions text"
                aria-atomic="false"
                aria-label={merged.customAriaLabel ?? merged.containerAriaLabel}
                tabIndex={-1}
                class={clsx(
                  toasterContainerClass,
                  getPositionClass(position),
                  expanded()
                    ? position.startsWith("top")
                      ? "flex flex-col"
                      : "flex flex-col-reverse"
                    : "grid",
                )}
                style={{
                  gap: `${merged.gap}px`,
                  ...offsetStyle(position),
                  ...(merged.style as Record<string, string | number>),
                }}
                onMouseEnter={() => setExpanded(true)}
                onMouseLeave={() => setExpanded(false)}
              >
                <For each={visibleToastsForPosition(position)}>
                  {(toast: ToastT, index) => (
                    <Toast
                      toast={toast}
                      index={index()}
                      total={visibleToastsForPosition(position).length}
                      expanded={expanded()}
                      position={position}
                      gap={merged.gap}
                      closeButton={
                        toast.closeButton ??
                        merged.toastOptions?.closeButton ??
                        merged.closeButton ??
                        false
                      }
                      duration={
                        merged.toastOptions?.duration ?? merged.duration
                      }
                      class={merged.toastOptions?.class}
                      icons={merged.icons}
                      closeButtonAriaLabel={
                        merged.toastOptions?.closeButtonAriaLabel
                      }
                      defaultRichColors={merged.richColors}
                      onRemove={removeToast}
                    />
                  )}
                </For>
              </ol>
            </Show>
          )}
        </For>
      </div>
    </Portal>
  );
};

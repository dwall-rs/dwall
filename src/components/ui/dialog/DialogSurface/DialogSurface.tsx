import {
  Show,
  createEffect,
  createSignal,
  onCleanup,
  splitProps,
} from "solid-js";
import type { DialogSurfaceProps } from "./DialogSurface.types";
import { DialogPortal } from "../DialogPortal";
import { DialogOverlay } from "../DialogOverlay";
import { DialogContentContext } from "../DialogContent/DialogContent.context";
import { useDialogContext } from "../Dialog";
import { callEventHandler } from "~/utils";
import { useDialogFocus } from "./useDialogFocus";

export function DialogSurface(props: DialogSurfaceProps) {
  const { show, open, setShow } = useDialogContext();

  const [local, events, others] = splitProps(
    props,
    [
      "role",
      "dismissOnOverlayClick",
      "overlayClass",
      "class",
      "classList",
      "children",
    ],
    ["onKeyDown"],
  );

  const { handleKeyDown } = useDialogFocus(open);

  const [titleID, setTitleID] = createSignal<string>();
  const [descriptionID, setDescriptionID] = createSignal<string>();
  let surfaceRef: HTMLElement | undefined;

  createEffect(() => {
    if (!open()) return;
    const frame = requestAnimationFrame(() => {
      surfaceRef?.focus({ preventScroll: true });
    });
    onCleanup(() => cancelAnimationFrame(frame));
  });

  return (
    <Show when={show()}>
      <DialogContentContext.Provider value={{ setTitleID, setDescriptionID }}>
        <DialogPortal>
          <div
            role="presentation"
            class="fixed inset-0 select-none"
            aria-hidden="true"
          />
          <DialogOverlay
            dismissOnOverlayClick={local.dismissOnOverlayClick}
            class={local.overlayClass}
          />
          <span data-type="inside" class="sr-only" aria-hidden="true" />
          {/* biome-ignore lint/a11y/useAriaPropsSupportedByRole: role 为 dialog/alertdialog，二者都支持 aria-labelledby/aria-describedby */}
          <div
            ref={(el) => {
              surfaceRef = el;
            }}
            role={local.role ?? "dialog"}
            aria-modal="true"
            tabindex={-1}
            data-dialog-surface=""
            data-open={open()}
            class={local.class}
            classList={local.classList}
            aria-labelledby={titleID()}
            aria-describedby={descriptionID()}
            onKeyDown={(event) => {
              callEventHandler(events.onKeyDown, event);
              if (!event.defaultPrevented) handleKeyDown(event);
            }}
            onAnimationEnd={() => {
              if (open()) return;
              setShow(false);
            }}
            {...others}
          >
            {local.children}
          </div>
          <span data-type="inside" class="sr-only" aria-hidden="true" />
        </DialogPortal>
      </DialogContentContext.Provider>
    </Show>
  );
}

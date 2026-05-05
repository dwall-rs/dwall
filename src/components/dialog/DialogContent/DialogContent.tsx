import { createSignal, mergeProps, Show, splitProps } from "solid-js";
import type { DialogContentProps } from "./DialogContent.types";
import { clsx } from "~/utils";
import { classes } from "./DialogContent.styles";
import { DialogPortal } from "../DialogPortal";
import { DialogOverlay } from "../DialogOverlay";
import { DialogContentContext } from "./DialogContent.context";
import { useDialogContext } from "../Dialog";
import { DialogClose } from "../DialogClose";

export const DialogContent = (props: DialogContentProps) => {
  const { show, open, setShow } = useDialogContext();

  const merged = mergeProps({ showCloseButton: true } as const, props);

  const [local, others] = splitProps(merged, [
    "showCloseButton",
    "class",
    "classList",
    "children",
  ]);

  const [titleID, setTitleID] = createSignal<string>();
  const [descriptionID, setDescriptionID] = createSignal<string>();

  return (
    <Show when={show()}>
      <DialogContentContext.Provider value={{ setTitleID, setDescriptionID }}>
        <DialogPortal>
          <div
            role="presentation"
            class="fixed inset-0 select-none"
            aria-hidden="true"
          />
          <DialogOverlay />
          <span data-type="inside" class="sr-only" aria-hidden="true" />
          <div
            data-slot="dialog-content"
            data-open={open()}
            class={clsx(classes, local.class)}
            role="dialog"
            aria-labelledby={titleID()}
            aria-describedby={descriptionID()}
            onAnimationEnd={() => {
              if (open()) return;
              setShow(false);
            }}
            {...others}
          >
            {local.children}
            <Show when={local.showCloseButton}>
              <DialogClose />
            </Show>
          </div>
          <span data-type="inside" class="sr-only" aria-hidden="true" />
        </DialogPortal>
      </DialogContentContext.Provider>
    </Show>
  );
};

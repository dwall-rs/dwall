import { createSignal, mergeProps, Show, splitProps } from "solid-js";
import type { AlertDialogContentProps } from "./AlertDialogContent.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogContent.styles";
import { AlertDialogPortal } from "../AlertDialogPortal";
import { AlertDialogOverlay } from "../AlertDialogOverlay";
import { AlertDialogContentContext } from "./AlertDialogContent.context";
import { useAlertDialogContext } from "../AlertDialog";

export const AlertDialogContent = (props: AlertDialogContentProps) => {
  const { show, open, setShow } = useAlertDialogContext();

  const merged = mergeProps({ size: "md" } as const, props);

  const [local, others] = splitProps(merged, ["size", "class", "classList"]);

  const [titleID, setTitleID] = createSignal<string>();
  const [descriptionID, setDescriptionID] = createSignal<string>();

  return (
    <Show when={show()}>
      <AlertDialogContentContext.Provider
        value={{ setTitleID, setDescriptionID }}
      >
        <AlertDialogPortal>
          <div
            role="presentation"
            class="fixed inset-0 select-none"
            aria-hidden="true"
          />
          <AlertDialogOverlay />
          <span
            data-type="inside"
            class="[clip-path:inset(50%)] overflow-hidden whitespace-nowrap border-none p-0 size-px -m-px fixed top-0 left-0"
            aria-hidden="true"
          />
          <div
            data-slot="alert-dialog-content"
            data-size={local.size}
            data-open={open()}
            class={clsx(classes, local.class)}
            role="alertdialog"
            aria-labelledby={titleID()}
            aria-describedby={descriptionID()}
            onAnimationEnd={() => {
              if (open()) return;
              setShow(false);
            }}
            {...others}
          />
          <span
            data-type="inside"
            class="[clip-path:inset(50%)] overflow-hidden whitespace-nowrap border-none p-0 size-px -m-px fixed top-0 left-0"
            aria-hidden="true"
          />
        </AlertDialogPortal>
      </AlertDialogContentContext.Provider>
    </Show>
  );
};

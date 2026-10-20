import { mergeProps, splitProps } from "solid-js";
import type { DialogOverlayProps } from "./DialogOverlay.types";
import { clsx } from "~/utils";
import { classes } from "./DialogOverlay.styles";
import { useDialogContext } from "../Dialog";

export const DialogOverlay = (props: DialogOverlayProps) => {
  const merged = mergeProps({ dismissOnOverlayClick: true } as const, props);

  const [local, others] = splitProps(merged, [
    "class",
    "classList",
    "dismissOnOverlayClick",
    "onClick",
  ]);

  const { open, setShow, setOpen } = useDialogContext();

  return (
    <div
      data-slot="dialog-overlay"
      role="presentation"
      aria-hidden="true"
      data-open={open()}
      class={clsx(classes, local.class)}
      classList={local.classList}
      onClick={(
        event: MouseEvent & {
          currentTarget: HTMLDivElement;
          target: Element;
        },
      ) => {
        const userOnClick = local.onClick as
          | ((e: typeof event) => void)
          | undefined;
        userOnClick?.(event);
        if (local.dismissOnOverlayClick) setOpen(false);
      }}
      onAnimationEnd={() => {
        if (open()) return;
        setShow(false);
      }}
      {...others}
    />
  );
};

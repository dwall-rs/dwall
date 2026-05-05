import { mergeProps, Show } from "solid-js";
import type { DialogCloseProps } from "./DialogClose.types";
import { Button } from "~/components/button";
import { clsx } from "~/utils";
import { classes } from "./DialogClose.styles";
import { X } from "lucide-solid";
import { useDialogContext } from "../Dialog";
import type {
  ButtonProps,
  ResolvedButtonProps,
} from "~/components/button/Button.types";

export const DialogClose = (props: DialogCloseProps) => {
  const { setOpen } = useDialogContext();

  const merged = mergeProps(
    {
      variant: "ghost",
      size: "sm",
    } as const,
    props as ResolvedButtonProps<"button">,
  );

  const handleClick = () => {
    setOpen(false);
    merged.onClick?.();
  };

  return (
    <Show
      when={!merged.children && !merged.icon}
      fallback={
        <Button
          {...(merged as ButtonProps<"button">)}
          data-slot="dialog-close"
          onClick={handleClick}
        />
      }
    >
      <Button
        {...(merged as ButtonProps<"button">)}
        data-slot="dialog-close"
        class={clsx(classes)}
        onClick={handleClick}
        icon={{ icon: <X />, ariaLabel: "Close" }}
      />
    </Show>
  );
};

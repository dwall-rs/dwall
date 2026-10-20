import { mergeProps, splitProps } from "solid-js";
import type { AlertDialogCancelProps } from "./AlertDialogCancel.types";
import { Button } from "~/components/ui/button";
import { useDialogContext } from "~/components/ui/dialog";

export const AlertDialogCancel = (props: AlertDialogCancelProps) => {
  const { setOpen } = useDialogContext();

  const merged = mergeProps({ variant: "outline", size: "md" } as const, props);
  const [local, others] = splitProps(merged, ["onClick"]);

  return (
    <Button
      {...others}
      data-slot="alert-dialog-cancel"
      onClick={(event) => {
        // 与 AlertDialogAction 一致：先转发调用方回调再关闭，
        // 否则用户的 onClick 会被这一行静默替换掉
        local.onClick?.(event);
        setOpen(false);
      }}
    />
  );
};

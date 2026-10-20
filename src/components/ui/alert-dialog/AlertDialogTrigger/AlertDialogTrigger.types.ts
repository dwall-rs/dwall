import type { ValidComponent } from "solid-js";
import type { Button } from "~/components/ui/button";
import type { DialogTriggerProps } from "~/components/ui/dialog";

export type AlertDialogTriggerProps<
  T extends ValidComponent = typeof Button<"button">,
> = DialogTriggerProps<T>;

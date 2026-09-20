import { splitProps } from "solid-js";
import type { InputGroupTextareaProps } from "./InputGroupTextarea.types";
import { Textarea } from "~/components/ui/textarea";
import { clsx } from "~/utils";

export const InputGroupTextarea = (props: InputGroupTextareaProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Textarea
      data-slot="input-group-control"
      class={clsx(
        "flex-1 resize-none rounded-none border-0 bg-transparent py-2 shadow-none ring-0 focus-visible:ring-0 disabled:bg-transparent aria-invalid:ring-0 dark:bg-transparent dark:disabled:bg-transparent",
        local.class,
      )}
      {...others}
    />
  );
};

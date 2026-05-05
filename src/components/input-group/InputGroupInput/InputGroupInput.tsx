import { Input } from "~/components/input/Input";
import type { InputGroupInputProps } from "./InputGroupInput.types";
import { splitProps } from "solid-js";
import { clsx } from "~/utils";

export const InputGroupInput = (props: InputGroupInputProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Input
      data-slot="input-group-control"
      class={clsx(
        "flex-1 rounded-none border-0 bg-transparent shadow-none ring-0 focus-visible:ring-0 disabled:bg-transparent aria-invalid:ring-0 dark:bg-transparent dark:disabled:bg-transparent",
        local.class,
      )}
      {...others}
    />
  );
};

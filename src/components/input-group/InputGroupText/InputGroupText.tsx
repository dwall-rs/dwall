import { splitProps } from "solid-js";
import type { InputGroupTextProps } from "./InputGroupText.types";
import { clsx } from "~/utils";

export const InputGroupText = (props: InputGroupTextProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <span
      class={clsx(
        "flex items-center gap-2 text-sm text-neutral-500 dark:text-neutral-400 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4",
        local.class,
      )}
      {...others}
    />
  );
};

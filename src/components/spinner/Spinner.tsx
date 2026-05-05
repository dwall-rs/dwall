import { LoaderCircle } from "lucide-solid";
import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import { classes } from "./Spinner.styles";
import type { SpinnerProps } from "./Spinner.types";

export const Spinner = (props: SpinnerProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <LoaderCircle
      class={clsx(classes, local.class)}
      role="status"
      aria-label="Loading"
      {...others}
    />
  );
};

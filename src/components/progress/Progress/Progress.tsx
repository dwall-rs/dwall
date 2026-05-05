import { splitProps } from "solid-js";
import type { ProgressProps } from "./Progress.types";
import { clsx } from "~/utils";
import { classes } from "./Progress.styles";
import { ProgressTrack } from "../ProgressTrack";
import { ProgressIndicator } from "../ProgressIndicator";
import { ProgressContext } from "./Progress.context";

export const Progress = (props: ProgressProps) => {
  const [local, others] = splitProps(props, [
    "value",
    "class",
    "classList",
    "children",
  ]);

  const value = () => local.value;

  return (
    <div
      data-slot="progress"
      aria-valuemax="100"
      aria-valuemin="0"
      aria-valuenow={local.value}
      aria-valuetext={`${local.value}%`}
      role="progressbar"
      data-progressing={local.value > 0 && local.value < 100}
      class={clsx(classes, local.class)}
      {...others}
    >
      <ProgressContext.Provider
        value={{
          value,
        }}
      >
        {local.children}
        <ProgressTrack>
          <ProgressIndicator />
        </ProgressTrack>
      </ProgressContext.Provider>
      <span
        role="presentation"
        class="[clip-path:inset(50%)] overflow-hidden whitespace-nowrap border-0 p-0 size-px -m-px fixed top-0 left-0"
      >
        x
      </span>
    </div>
  );
};

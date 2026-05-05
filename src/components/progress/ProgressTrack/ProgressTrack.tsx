import { splitProps } from "solid-js";
import type { ProgressTrackProps } from "./ProgressTrack.types";
import { clsx } from "~/utils";
import { classes } from "./ProgressTrack.styles";
import { useProgressContext } from "../Progress";

export const ProgressTrack = (props: ProgressTrackProps) => {
  const { value } = useProgressContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="progress-track"
      data-progressing={value() > 0 && value() < 100}
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};

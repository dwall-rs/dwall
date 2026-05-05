import { splitProps } from "solid-js";
import type { SkeletonProps } from "./Skeleton.types";
import { clsx } from "~/utils";

export const Skeleton = (props: SkeletonProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="skeleton"
      class={clsx(
        "animate-pulse rounded-md bg-neutral-100 dark:bg-neutral-800",
        local.class,
      )}
      {...others}
    />
  );
};

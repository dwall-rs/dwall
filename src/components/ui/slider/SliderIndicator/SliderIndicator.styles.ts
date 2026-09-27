import { clsx } from "~/utils";

export const classes = clsx(
  "bg-primary select-none",
  "data-[orientation=horizontal]:relative data-[orientation=horizontal]:h-full data-[orientation=horizontal]:inset-s-(--start-position) data-[orientation=horizontal]:w-(--relative-size)",
  "data-[orientation=vertical]:absolute data-[orientation=vertical]:w-full data-[orientation=vertical]:h-(--start-position) data-[orientation=vertical]:bottom-(--relative-size)",
  // "bg-primary select-none data-horizontal:h-full data-vertical:w-full",
);

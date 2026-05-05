import { clsx } from "~/utils";

export const classes = clsx(
  "flex h-9 w-full items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm shadow-xs",
  "transition-[color,box-shadow] outline-none",
  "border-neutral-200 dark:border-white/15 bg-transparent text-foreground",
  "focus-visible:border-neutral-400 dark:focus-visible:border-neutral-500 focus-visible:ring-3 focus-visible:ring-neutral-400/50 dark:focus-visible:ring-neutral-500/50",
  "disabled:pointer-events-none disabled:cursor-not-allowed disabled:select-none disabled:bg-neutral-200/50 disabled:opacity-50",
  "dark:bg-white/4.5 dark:disabled:bg-white/12",
);

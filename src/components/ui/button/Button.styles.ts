import { cva } from "class-variance-authority";
import { clsx } from "~/utils";

export const base =
  "group/button inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:border-neutral-400 focus-visible:ring-3 focus-visible:ring-neutral-400/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-red-600 aria-invalid:ring-3 aria-invalid:ring-red-600/20 dark:aria-invalid:border-red-600/50 dark:aria-invalid:ring-red-600/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

export default cva(base, {
  variants: {
    variant: {
      primary: clsx(
        "bg-neutral-900 text-neutral-50 [a]:hover:bg-neutral-900/80",
        "dark:bg-neutral-200 dark:text-neutral-900 [a]:hover:bg-neutral-200/80",
      ),
      outline: clsx(
        "border-neutral-200 bg-white hover:bg-neutral-100 hover:text-neutral-950 aria-expanded:bg-neutral-100 aria-expanded:text-neutral-950",
        "dark:border-white/15 dark:bg-white/4 dark:hover:bg-white/5 dark:hover:text-neutral-50 dark:aria-expanded:bg-neutral-800 dark:aria-expanded:text-neutral-50",
      ),
      secondary: clsx(
        "bg-neutral-100 text-neutral-900 hover:bg-neutral-100/80 aria-expanded:bg-neutral-100 aria-expanded:text-neutral-900",
        "dark:bg-neutral-800 dark:text-neutral-50 dark:hover:bg-neutral-800/80 aria-expanded:bg-neutral-800 aria-expanded:text-neutral-50",
      ),
      ghost: clsx(
        "hover:bg-neutral-100 hover:text-neutral-950 aria-expanded:bg-neutral-100 aria-expanded:text-neutral-950",
        "dark:hover:bg-neutral-800/50 dark:hover:text-neutral-50 dark:aria-expanded:bg-neutral-800 dark:aria-expanded:text-neutral-50",
      ),
      destructive: clsx(
        "bg-red-600/10 text-red-600 hover:bg-red-600/20 focus-visible:border-red-600/40 focus-visible:ring-red-600/20",
        "dark:bg-red-400/20 dark:text-red-400 dark:hover:bg-red-400/30 dark:focus-visible:border-red-400/40 dark:focus-visible:ring-red-400/40",
      ),
      link: clsx(
        "underline-offset-4 hover:underline",
        "text-neutral-900",
        "dark:text-neutral-200",
      ),
    },
    size: {
      md: "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
      xs: "h-6 gap-1 rounded-[min(var(--radius-md),10px)] px-2 text-xs in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3",
      sm: "h-7 gap-1 rounded-[min(var(--radius-md),12px)] px-2.5 text-[0.8rem] in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3.5",
      lg: "h-9 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
    },
    iconSize: {
      md: "size-8",
      xs: "size-6 rounded-[min(var(--radius-md),10px)] in-data-[slot=button-group]:rounded-lg [&_svg:not([class*='size-'])]:size-3",
      sm: "size-7 rounded-[min(var(--radius-md),12px)] in-data-[slot=button-group]:rounded-lg",
      lg: "size-9",
    },
  },
});

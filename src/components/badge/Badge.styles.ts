import { cva } from "class-variance-authority";
import { clsx } from "~/utils";

export const badgeVariants = cva(
  "group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 aria-invalid:border-red-600 aria-invalid:ring-red-600/20 dark:aria-invalid:ring-red-600/40 [&>svg]:pointer-events-none [&>svg]:size-3! dark:aria-invalid:border-red-600 dark:aria-invalid:ring-red-400/40 ",
  {
    variants: {
      variant: {
        primary: clsx(
          "bg-neutral-900 text-neutral-50 [a]:hover:bg-neutral-900/80",
          "dark:bg-neutral-200 dark:text-neutral-900 dark:[a]:hover:bg-neutral-200/80",
        ),
        secondary: clsx(
          "bg-neutral-100 text-neutral-900 [a]:hover:bg-neutral-100/80",
          "dark:bg-neutral-800 dark:text-neutral-50 dark:[a]:hover:bg-neutral-800/80",
        ),
        destructive: clsx(
          "bg-red-600/10 text-red-600 focus-visible:ring-red-600/20 [a]:hover:bg-red-600/20",
          "dark:bg-red-400/20 dark:text-red-400 dark:focus-visible:ring-red-400/40 dark:[a]:hover:bg-red-400/20",
        ),
        outline: clsx(
          "border-neutral-200 text-neutral-950 [a]:hover:bg-neutral-100 [a]:hover:text-neutral-500",
          "dark:border-white/10 dark:text-neutral-50 dark:[a]:hover:bg-neutral-800 dark:[a]:hover:text-neutral-400",
        ),
        ghost: clsx(
          "hover:bg-neutral-100 hover:text-neutral-500",
          "dark:hover:text-neutral-400 dark:hover:bg-neutral-800/50",
        ),
        link: clsx(
          "text-neutral-900 underline-offset-4 hover:underline",
          "dark:text-neutral-200",
        ),
      },
    },
    defaultVariants: {
      variant: "primary",
    },
  },
);

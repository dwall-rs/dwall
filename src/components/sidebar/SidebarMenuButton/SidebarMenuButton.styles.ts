import { cva } from "class-variance-authority";
import { clsx } from "~/utils";

export const sidebarMenuButtonVariants = cva(
  clsx(
    "peer/menu-button group/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm outline-hidden cursor-default transition-[width,height,padding] group-has-data-[sidebar=menu-action]/menu-item:pr-8 group-data-[collapsible=icon]:size-8! group-data-[collapsible=icon]:p-2! focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[active=true]:font-medium [&_svg]:size-4 [&_svg]:shrink-0 [&>span:last-child]:truncate",
    "ring-neutral-400 hover:bg-neutral-100 hover:text-neutral-900 active:bg-neutral-100 active:text-neutral-900 data-open:hover:bg-neutral-100 data-open:hover:text-neutral-900 data-[active=true]:bg-neutral-100 data-[active=true]:text-neutral-900",
    "dark:ring-neutral-500 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:active:bg-neutral-800 dark:active:text-neutral-50 dark:data-open:hover:bg-neutral-800 dark:data-open:hover:text-neutral-50 dark:data-[active=true]:bg-neutral-800 dark:data-[active=true]:text-neutral-50",
    "data-[active=true]:translate-y-px active:translate-y-px transition-[translate]",
  ),
  {
    variants: {
      variant: {
        ghost:
          "hover:bg-neutral-100 hover:text-neutral-900 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 active:bg-neutral-100/50",
        outline: clsx(
          "bg-white shadow-[0_0_0_1px_(var(--color-neutral-200))] hover:bg-neutral-100 hover:text-neutral-900 hover:shadow-[0_0_0_1px_var(--color-neutral-100)]",
          "dark:bg-neutral-950 dark:shadow-[0_0_0_1px_hsl(var(--sidebar-border))] dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:hover:shadow-[0_0_0_1px_--alpha(var(--color-white) / 10%)]]",
        ),
      },
      size: {
        md: "h-8 text-sm",
        sm: "h-7 text-xs",
        lg: "h-12 text-sm group-data-[collapsible=icon]:p-0!",
      },
    },
    defaultVariants: {
      variant: "ghost",
      size: "md",
    },
  },
);

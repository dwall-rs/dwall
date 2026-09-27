import { cva } from "class-variance-authority";

export const toastVariants = cva(
  "pointer-events-auto relative flex w-full items-center gap-2.5 rounded-lg border bg-popover p-4 text-sm text-popover-foreground shadow-md outline-none",
  {
    variants: {
      type: {
        default: "border-border",
        success: "border-emerald-500/60",
        info: "border-blue-500/60",
        warning: "border-amber-500/60",
        error: "border-red-500/60",
        loading: "border-border",
      },
    },
    defaultVariants: {
      type: "default",
    },
  },
);

export const toastIconVariants = cva(
  "flex size-5 shrink-0 items-center justify-center",
  {
    variants: {
      type: {
        default: "text-muted-foreground",
        success: "text-emerald-500",
        info: "text-blue-500",
        warning: "text-amber-500",
        error: "text-red-500",
        loading: "text-muted-foreground",
      },
    },
    defaultVariants: {
      type: "default",
    },
  },
);

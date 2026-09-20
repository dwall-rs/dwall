import { cva } from "class-variance-authority";

export const inputGroupButtonVariants = cva(
  "flex items-center gap-2 text-sm shadow-none",
  {
    variants: {
      size: {
        xs: "h-6 gap-1 rounded-[calc(var(--radius)-3px)] px-1.5 [&>svg:not([class*='size-'])]:size-3.5",
        sm: "",
      },
      iconSize: {
        xs: "size-6 rounded-[calc(var(--radius)-3px)] p-0 has-[>svg]:p-0",
        sm: "size-8 p-0 has-[>svg]:p-0",
      },
    },
    defaultVariants: {
      size: "xs",
    },
  },
);

import { cva } from "class-variance-authority";

const toastBase =
  "pointer-events-auto relative flex w-full items-center gap-2 rounded-lg bg-popover px-3 py-2.5 text-sm text-popover-foreground shadow-md ring-1 ring-foreground/10 outline-hidden duration-200";

const semanticSuccess =
  "bg-[color-mix(in_oklab,var(--popover),var(--success)_10%)] text-success [&_[data-slot=toast-description]]:text-success/80";
const semanticInfo =
  "bg-[color-mix(in_oklab,var(--popover),var(--info)_10%)] text-info [&_[data-slot=toast-description]]:text-info/80";
const semanticWarning =
  "bg-[color-mix(in_oklab,var(--popover),var(--warning)_10%)] text-warning [&_[data-slot=toast-description]]:text-warning/80";
const semanticDestructive =
  "bg-[color-mix(in_oklab,var(--popover),var(--destructive)_10%)] text-destructive [&_[data-slot=toast-description]]:text-destructive/80";

const coloredSuccess =
  "bg-success text-success-foreground [&_[data-slot=toast-icon]]:text-success-foreground [&_[data-slot=toast-description]]:text-success-foreground/80 [&_[data-slot=toast-close]]:text-success-foreground/80 [&_[data-slot=toast-close]]:hover:text-success-foreground";
const coloredInfo =
  "bg-info text-info-foreground [&_[data-slot=toast-icon]]:text-info-foreground [&_[data-slot=toast-description]]:text-info-foreground/80 [&_[data-slot=toast-close]]:text-info-foreground/80 [&_[data-slot=toast-close]]:hover:text-info-foreground";
const coloredWarning =
  "bg-warning text-warning-foreground [&_[data-slot=toast-icon]]:text-warning-foreground [&_[data-slot=toast-description]]:text-warning-foreground/80 [&_[data-slot=toast-close]]:text-warning-foreground/80 [&_[data-slot=toast-close]]:hover:text-warning-foreground";
const coloredDestructive =
  "bg-destructive text-destructive-foreground [&_[data-slot=toast-icon]]:text-destructive-foreground [&_[data-slot=toast-description]]:text-destructive-foreground/80 [&_[data-slot=toast-close]]:text-destructive-foreground/80 [&_[data-slot=toast-close]]:hover:text-destructive-foreground";
const coloredNeutral =
  "bg-primary text-primary-foreground [&_[data-slot=toast-icon]]:text-primary-foreground [&_[data-slot=toast-description]]:text-primary-foreground/80 [&_[data-slot=toast-close]]:text-primary-foreground/80 [&_[data-slot=toast-close]]:hover:text-primary-foreground";

export const toastVariants = cva(toastBase, {
  variants: {
    variant: {
      popover: "",
      semantic: "",
      colored: "",
    },
    type: {
      default: "",
      success: "",
      info: "",
      warning: "",
      error: "",
      loading: "",
    },
  },
  compoundVariants: [
    { variant: "semantic", type: "success", class: semanticSuccess },
    { variant: "semantic", type: "info", class: semanticInfo },
    { variant: "semantic", type: "warning", class: semanticWarning },
    { variant: "semantic", type: "error", class: semanticDestructive },
    { variant: "colored", type: "default", class: coloredNeutral },
    { variant: "colored", type: "loading", class: coloredNeutral },
    { variant: "colored", type: "success", class: coloredSuccess },
    { variant: "colored", type: "info", class: coloredInfo },
    { variant: "colored", type: "warning", class: coloredWarning },
    { variant: "colored", type: "error", class: coloredDestructive },
  ],
  defaultVariants: {
    variant: "popover",
    type: "default",
  },
});

export const toastIconVariants = cva(
  "flex size-4 shrink-0 items-center justify-center [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      type: {
        default: "text-muted-foreground",
        success: "text-success",
        info: "text-info",
        warning: "text-warning",
        error: "text-destructive",
        loading: "text-muted-foreground",
      },
    },
    defaultVariants: {
      type: "default",
    },
  },
);

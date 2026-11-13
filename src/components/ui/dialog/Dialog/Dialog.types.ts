import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseDialogProps {
  open?: boolean;

  defaultOpen?: boolean;

  onOpenChange?: (open: boolean) => void;

  dismissOnEscape?: boolean;

  lockScroll?: boolean;
}

export type DialogProps = PolymorphicProps<
  "div",
  BaseProps & BaseDialogProps,
  false
>;

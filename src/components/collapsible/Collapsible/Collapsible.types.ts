import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseCollapsibleProps {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export type CollapsibleProps = PolymorphicProps<
  "div",
  BaseProps & BaseCollapsibleProps,
  false
>;

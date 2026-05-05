import type { VariantProps } from "class-variance-authority";
import type { TooltipContentProps } from "~/components/tooltip/TooltipContent/TooltipContent.types";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { sidebarMenuButtonVariants } from "./SidebarMenuButton.styles";
import type { ValidComponent } from "solid-js";

export type SidebarMenuButtonProps<T extends ValidComponent> = PolymorphicProps<
  T,
  BaseProps & {
    isActive?: boolean;
    disabled?: boolean;
    tooltip?: string | TooltipContentProps;
  } & VariantProps<typeof sidebarMenuButtonVariants>
>;

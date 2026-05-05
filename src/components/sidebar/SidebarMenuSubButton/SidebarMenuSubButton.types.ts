import type { ValidComponent } from "solid-js";
import type { BaseProps, PolymorphicProps } from "~/types";

export type SidebarMenuSubButtonProps<T extends ValidComponent> =
  PolymorphicProps<
    T,
    BaseProps & {
      size?: "sm" | "md";
      isActive?: boolean;
    },
    false
  >;

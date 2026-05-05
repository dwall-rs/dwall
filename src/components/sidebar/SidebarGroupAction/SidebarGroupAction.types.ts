import type { ValidComponent } from "solid-js";
import type { BaseProps, PolymorphicProps } from "~/types";

export type SidebarGroupActionProps<T extends ValidComponent> =
  PolymorphicProps<T, BaseProps>;

import type { ValidComponent } from "solid-js";
import type { PolymorphicProps } from "~/types";

export type TooltipTriggerProps<T extends ValidComponent> = PolymorphicProps<T>;

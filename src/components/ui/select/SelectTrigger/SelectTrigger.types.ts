import type { ValidComponent } from "solid-js";
import type { PolymorphicProps } from "~/types";

export type SelectTriggerProps<T extends ValidComponent> = PolymorphicProps<T>;

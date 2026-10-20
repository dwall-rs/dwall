import type { ValidComponent } from "solid-js";
import type { PolymorphicProps } from "~/types";

export type DialogTriggerProps<T extends ValidComponent> = PolymorphicProps<T>;

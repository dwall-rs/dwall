import type { Label } from "~/components/label";
import type { BaseProps, PolymorphicProps } from "~/types";

export type FieldLabelProps = PolymorphicProps<typeof Label, BaseProps, false>;

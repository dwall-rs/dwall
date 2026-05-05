import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { badgeVariants } from "./Badge.styles";

type BaseBadgeProps = VariantProps<typeof badgeVariants> & BaseProps;

export type BadgeProps = PolymorphicProps<"span", BaseBadgeProps, false>;

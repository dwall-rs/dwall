import type { VariantProps } from "class-variance-authority";
import type { inputGroupButtonVariants } from "./InputGroupButton.styles";
import type { ButtonProps } from "~/components/ui/button";

export type InputGroupButtonProps = Omit<ButtonProps<"button">, "size"> &
  Pick<VariantProps<typeof inputGroupButtonVariants>, "size">;

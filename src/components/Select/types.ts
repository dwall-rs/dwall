import type { JSX } from "solid-js";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  label?: string;
  value: string;
  onChange?: (value: string) => void;
  options: SelectOption[];
  disabled?: boolean;
  required?: boolean;
  placeholder?: string;
  style?: JSX.CSSProperties;
  size?: "small" | "medium" | "large";
  appearance?: "outline" | "underline" | "filled-darker" | "filled-lighter";
  autofocus?: boolean;
}

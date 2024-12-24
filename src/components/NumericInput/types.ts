import type { InputProps } from "fluent-solid/lib/components/input";

export interface NumericInputProps {
  label?: string;
  suffix?: string;
  value?: number;
  onChange?: (value?: number) => void;
  disabled?: boolean;
  required?: boolean;
  min?: number;
  max?: number;
  size?: InputProps["size"];
  appearance?: InputProps["appearance"];
  placeholder?: InputProps["placeholder"];
  style?: InputProps["style"];
  autofocus?: boolean;
}

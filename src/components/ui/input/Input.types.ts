import type { JSX } from "solid-js";

type InputType =
  | "button"
  | "checkbox"
  | "color"
  | "date"
  | "datetime-local"
  | "email"
  | "file"
  | "hidden"
  | "image"
  | "month"
  | "number"
  | "password"
  | "radio"
  | "range"
  | "reset"
  | "search"
  | "submit"
  | "tel"
  | "text"
  | "time"
  | "url"
  | "week";

type NativeInputProps = JSX.IntrinsicElements["input"];

export interface NumberInputSpecifics {
  type: "number";
  onChange?: (value: number) => void;
  onInput?: (value: number) => void;
  defaultValue?: number;
}

interface TextInputSpecifics {
  type?: Exclude<InputType, "number">;
  onChange?: (value: string) => void;
  onInput?: (value: string) => void;
  defaultValue?: string;
}

export type InputProps =
  | (NumberInputSpecifics & Omit<NativeInputProps, keyof NumberInputSpecifics>)
  | (TextInputSpecifics & Omit<NativeInputProps, keyof TextInputSpecifics>);

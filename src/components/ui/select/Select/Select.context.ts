import { createContext, useContext } from "solid-js";
import type { SelectContextValue, SelectOptionValue } from "./Select.types";

export const SelectContext = createContext<SelectContextValue>();

export function useSelectContext(
  component: string,
): SelectContextValue<SelectOptionValue> {
  const ctx = useContext(SelectContext);
  if (!ctx) {
    throw new Error(`<${component}> must be rendered inside <Select>`);
  }
  return ctx;
}

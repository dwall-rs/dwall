import { createContext, useContext } from "solid-js";

interface ProgressContextValue {
  value: () => number;
}

export const ProgressContext = createContext<ProgressContextValue>();

export const useProgressContext = () => {
  const context = useContext(ProgressContext);
  if (!context) {
    throw new Error(
      "useProgressContext must be used within a ProgressProvider",
    );
  }
  return context;
};

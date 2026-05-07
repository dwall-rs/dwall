import {
  type Accessor,
  createContext,
  type Setter,
  useContext,
} from "solid-js";

interface CollapsibleContextValue {
  open: Accessor<boolean>;
  setInternalOpen: Setter<boolean>;
  onOpenChange?: (open: boolean) => void;
}

export const CollapsibleContext = createContext<CollapsibleContextValue>();

export const useCollapsibleContext = () => {
  const ctx = useContext(CollapsibleContext);
  if (!ctx) {
    throw new Error(
      "useCollapsibleContext must be used within a CollapsibleProvider",
    );
  }
  return ctx;
};

import { createMemo, createSignal, splitProps } from "solid-js";
import type { CollapsibleProps } from "./Collapsible.types";
import { CollapsibleContext } from "./Collapsible.context";

export const Collapsible = (props: CollapsibleProps) => {
  const [local, others] = splitProps(props, [
    "defaultOpen",
    "open",
    "onOpenChange",
    "children",
  ]);

  const [internalOpen, setInternalOpen] = createSignal(
    local.defaultOpen || false,
  );

  const open = createMemo(() =>
    local.open === undefined ? internalOpen() : local.open,
  );

  return (
    <div data-slot="collapsible" data-open={open()} {...others}>
      <CollapsibleContext.Provider
        value={{
          open,
          setInternalOpen,
          onOpenChange: local.onOpenChange,
        }}
      >
        {local.children}
      </CollapsibleContext.Provider>
    </div>
  );
};

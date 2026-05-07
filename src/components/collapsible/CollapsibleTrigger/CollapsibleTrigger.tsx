import { useCollapsibleContext } from "../Collapsible";
import type { CollapsibleTriggerProps } from "./CollapsibleTrigger.types";
import { Button } from "~/components/button";

export const CollapsibleTrigger = (props: CollapsibleTriggerProps) => {
  const { onOpenChange, open, setInternalOpen } = useCollapsibleContext();

  const handleClick = () => {
    const next = !open();
    setInternalOpen(next);
    onOpenChange?.(next);
  };

  return (
    // @ts-expect-error
    <Button
      {...props}
      data-slot="collapsible-trigger"
      data-panel-open={open()}
      aria-expanded={open()}
      onClick={handleClick}
    />
  );
};

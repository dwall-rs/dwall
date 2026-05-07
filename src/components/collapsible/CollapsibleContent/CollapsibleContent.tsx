import { Show, splitProps } from "solid-js";
import type { CollapsibleContentProps } from "./CollapsibleContent.types";
import { useCollapsibleContext } from "../Collapsible";

export const CollapsibleContent = (props: CollapsibleContentProps) => {
  const { open } = useCollapsibleContext();

  const [local, others] = splitProps(props, ["children"]);

  return (
    <Show when={open()}>
      <div data-slot="collapsible-content" {...others}>
        {local.children}
      </div>
    </Show>
  );
};

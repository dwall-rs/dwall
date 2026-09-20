import { splitProps } from "solid-js";
import { Dynamic } from "solid-js/web";
import { useSelectTrigger } from "./useSelectTrigger";
import type { SelectTriggerProps } from "./SelectTrigger.types";
import { Button } from "../../button";

function mergeRefs(
  ...refs: Array<Element | ((el: Element) => void) | undefined>
) {
  return (el: Element) => {
    for (const ref of refs) {
      if (typeof ref === "function") ref(el);
    }
  };
}

export function SelectTrigger(props: SelectTriggerProps) {
  const { ctx, isDisabled, attachListeners } = useSelectTrigger(() => props);
  const tag = () => props.as ?? Button;

  const [local, rest] = splitProps(props, [
    "as",
    "class",
    "disabled",
    "ref",
    "children",
  ]);

  return (
    <Dynamic
      component={tag()}
      ref={mergeRefs(ctx.setReference, local.ref, attachListeners)}
      type={tag() === "button" ? "button" : undefined}
      disabled={tag() === "button" ? isDisabled() : undefined}
      role="combobox"
      aria-haspopup="listbox"
      aria-expanded={ctx.open()}
      aria-controls={ctx.open() ? ctx.contentId : undefined}
      data-state={ctx.open() ? "open" : "closed"}
      data-disabled={isDisabled() ? "" : undefined}
      class={local.class}
      {...rest}
    >
      {local.children}
    </Dynamic>
  );
}

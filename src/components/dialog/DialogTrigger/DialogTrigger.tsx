import { children, untrack, type JSXElement, type ParentProps } from "solid-js";
import { useDialogTrigger } from "./useDialogTrigger";

export const DialogTrigger = (props: ParentProps) => {
  const resolved = children(() => untrack(() => props.children));
  useDialogTrigger(resolved);

  return resolved as unknown as JSXElement;
};

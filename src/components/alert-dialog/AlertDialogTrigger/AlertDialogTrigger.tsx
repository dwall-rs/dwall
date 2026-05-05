import { children, untrack, type JSXElement, type ParentProps } from "solid-js";
import { useAlertDialogTrigger } from "./useAlertDialogTrigger";

export const AlertDialogTrigger = (props: ParentProps) => {
  const resolved = children(() => untrack(() => props.children));
  useAlertDialogTrigger(resolved);

  return resolved as unknown as JSXElement;
};

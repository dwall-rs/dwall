import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import classes from "./ButtonGroupText.styles";
import type { ButtonGroupTextProps } from "./ButtonGroupText.types";

export const ButtonGroupText = (props: ButtonGroupTextProps) => {
  // class / classList 必须摘出来：`class=` 写在 `{...props}` 之前时，
  // Solid 的 spread 会用 node.className 重设类名，把内置样式整体覆盖掉
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="button-group-text"
      class={clsx(classes, local.class)}
      classList={local.classList}
      {...others}
    />
  );
};

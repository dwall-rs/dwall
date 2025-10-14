import type { ButtonProps } from "fluent-solid";

import { LazyButton } from "~/lazy";

import { dangerButtonStyles } from "./DangerButton.css";

const DangerButton = (props: Omit<ButtonProps, "type">) => {
  return <LazyButton class={dangerButtonStyles} {...props} />;
};

export default DangerButton;

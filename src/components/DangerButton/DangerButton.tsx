import { LazyButton } from "~/lazy";
import { dangerButtonStyles } from "./DangerButton.css";
import type { ButtonProps } from "fluent-solid";

const DangerButton = (props: Omit<ButtonProps, "type">) => {
  return <LazyButton class={dangerButtonStyles} {...props} />;
};

export default DangerButton;

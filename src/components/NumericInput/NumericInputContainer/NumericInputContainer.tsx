import { Show, type Component } from "solid-js";
import type { JSX } from "solid-js";
import * as styles from "./NumericInputContainer.css";
import { LazyLabel } from "~/lazy";

const NumericInputContainer: Component<{
  children: JSX.Element;
  label?: string;
  required?: boolean;
  warning?: string;
}> = (props) => (
  <div class={styles.container}>
    <Show when={props.label}>
      <LazyLabel required={props.required}>{props.label}</LazyLabel>
    </Show>
    {props.children}
    <Show when={props.warning}>
      <div class={styles.warningMessage}>{props.warning}</div>
    </Show>
  </div>
);

export default NumericInputContainer;

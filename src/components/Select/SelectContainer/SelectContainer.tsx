import { Show, type Component } from "solid-js";
import { LazyLabel } from "~/lazy";
import type { SelectContainerProps } from "./SelectContainer.types";
import * as styles from "./SelectContainer.css";

const SelectContainer: Component<SelectContainerProps> = (props) => (
  <div class={styles.container}>
    <Show when={props.label}>
      <LazyLabel id={props.labelId} required={props.required}>
        {props.label}
      </LazyLabel>
    </Show>
    {props.children}
    <Show when={props.warning}>
      <div class={styles.warningMessage}>{props.warning}</div>
    </Show>
  </div>
);

export default SelectContainer;

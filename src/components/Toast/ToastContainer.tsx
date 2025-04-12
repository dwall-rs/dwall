import type { Component } from "solid-js";
import { Portal } from "solid-js/web";
import type { ToastContainerProps } from "./types";

/**
 * Toast container component - Manages toast position and animation effects
 */
const ToastContainer: Component<ToastContainerProps> = (props) => {
  return (
    <Portal
      ref={(el) => {
        el.classList.add("fluent-toast-container");
        el.classList.add(`fluent-toast-container-${props.position}`);
      }}
    >
      {props.children}
    </Portal>
  );
};

export default ToastContainer;

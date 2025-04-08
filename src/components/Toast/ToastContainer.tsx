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
      }}
    >
      <div
        class={`fluent-toast fluent-toast-${props.position}`}
        style={{
          ...props.style,
          "z-index": props.zIndex,
          "--fui-toast-max-width":
            typeof props.maxWidth === "number"
              ? `${props.maxWidth}px`
              : props.maxWidth,
        }}
        role="alert"
        aria-live="assertive"
      >
        {props.children}
      </div>
    </Portal>
  );
};

export default ToastContainer;

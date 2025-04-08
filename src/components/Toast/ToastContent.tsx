import type { Component } from "solid-js";
import type { ToastContentProps } from "./types";
import ToastIcon from "./ToastIcon";
import { LazyButton } from "~/lazy";
import { VsClose } from "solid-icons/vs";

/**
 * Toast content component - Renders toast content including icon, message and action buttons
 */
const ToastContent: Component<ToastContentProps> = (props) => {
  return (
    <div class={`fluent-toast-content fluent-toast-${props.type}`}>
      <div class="fluent-toast-icon-message">
        {props.icon || <ToastIcon type={props.type} />}
        <span class="fluent-toast-message">{props.message}</span>
      </div>
      <div class="fluent-toast-actions">
        {props.action}
        {props.closable && (
          <LazyButton
            onClick={props.onClose}
            appearance="subtle"
            size="small"
            icon={<VsClose />}
          />
        )}
      </div>
    </div>
  );
};

export default ToastContent;

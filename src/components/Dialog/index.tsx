import {
  createSignal,
  Show,
  splitProps,
  type Component,
  type JSX,
} from "solid-js";
import "./index.scss";
import { LazyButton, LazyDivider } from "~/lazy";

interface DialogProps {
  open?: boolean;
  style?: JSX.CSSProperties;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  title?: string;
  showMask?: boolean;
  showCloseButton?: boolean;
  maskClosable?: boolean;
  footer?: JSX.Element;
  children?: JSX.Element;
  onClose?: () => void;
}

const Dialog: Component<DialogProps> = (props) => {
  const [local, others] = splitProps(props, [
    "open",
    "defaultOpen",
    "onOpenChange",
    "title",
    "onClose",
    "showMask",
    "showCloseButton",
    "maskClosable",
    "footer",
    "children",
  ]);

  const [internalOpen, setInternalOpen] = createSignal(
    local.defaultOpen || false,
  );

  const isControlled = () => local.open !== undefined;

  const isOpen = () => (isControlled() ? local.open : internalOpen());

  const handleClose = () => {
    if (isControlled()) {
      local.onOpenChange?.(false);
    } else {
      setInternalOpen(false);
    }
    local.onClose?.();
  };

  const handleMaskClick = () => {
    if (local.maskClosable !== false) {
      handleClose();
    }
  };

  return (
    <Show when={isOpen()}>
      <div class="fluent-dialog-container">
        <Show when={local.showMask !== false}>
          <div class="fluent-dialog-mask" onClick={handleMaskClick} />
        </Show>

        <div class="fluent-dialog" {...others}>
          <Show when={local.title}>
            <div class="fluent-dialog-header">
              <h3 class="fluent-dialog-title">{local.title}</h3>
              <Show when={local.showCloseButton}>
                <LazyButton
                  class="fluent-dialog-close"
                  onClick={handleClose}
                  appearance="transparent"
                  shape="circular"
                  size="small"
                >
                  ×
                </LazyButton>
              </Show>
            </div>
            <LazyDivider />
          </Show>

          <div class="fluent-dialog-content">{local.children}</div>

          <Show when={local.footer}>
            <div class="fluent-dialog-footer">{local.footer}</div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

export default Dialog;

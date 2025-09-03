import {
  createSignal,
  Show,
  splitProps,
  type Component,
  type JSX,
} from "solid-js";

import { LazyButton, LazyDivider } from "~/lazy";

import styles from "./Dialog.css";

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
      <div class={styles.container}>
        <Show when={local.showMask !== false}>
          <div class={styles.mask} onClick={handleMaskClick} role="dialog" />
        </Show>

        <div class={styles.base} {...others}>
          <Show when={local.title}>
            <div class={styles.header}>
              <h3 class={styles.title}>{local.title}</h3>
              <Show when={local.showCloseButton}>
                <LazyButton
                  class={styles.close}
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

          <div class={styles.content}>{local.children}</div>

          <Show when={local.footer}>
            <div class={styles.footer}>{local.footer}</div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

export default Dialog;

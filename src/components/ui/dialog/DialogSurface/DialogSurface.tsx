import {
  Show,
  createEffect,
  createSignal,
  onCleanup,
  splitProps,
} from "solid-js";
import type { DialogSurfaceProps } from "./DialogSurface.types";
import { DialogPortal } from "../DialogPortal";
import { DialogOverlay } from "../DialogOverlay";
import { DialogContentContext } from "../DialogContent/DialogContent.context";
import { useDialogContext } from "../Dialog";

/** 浮层内可聚焦的元素，按文档顺序。 */
const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

export function DialogSurface(props: DialogSurfaceProps) {
  const { show, open, setShow, trigger } = useDialogContext();

  const [local, others] = splitProps(props, [
    "role",
    "dismissOnOverlayClick",
    "overlayClass",
    "class",
    "classList",
    "children",
  ]);

  const [titleID, setTitleID] = createSignal<string>();
  const [descriptionID, setDescriptionID] = createSignal<string>();
  let surfaceRef: HTMLElement | undefined;

  /** 浮层内当前可聚焦的元素；零尺寸的（隐藏的）节点不算。 */
  const focusableItems = (): HTMLElement[] => {
    const root = surfaceRef;
    if (!root) return [];

    return Array.from(
      root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR),
    ).filter((element) => element.getClientRects().length > 0);
  };

  /**
   * Tab 只在浮层内循环。
   *
   * aria-modal 只告诉读屏器忽略浮层外的内容，浏览器仍然允许 Tab 走到遮罩后面，
   * 越过最后一个控件后焦点就从视觉上消失了。
   */
  const trapFocus = (event: KeyboardEvent) => {
    if (event.key !== "Tab" || event.defaultPrevented) return;

    const items = focusableItems();
    if (items.length === 0) {
      event.preventDefault();
      surfaceRef?.focus({ preventScroll: true });
      return;
    }

    const first = items[0];
    const last = items[items.length - 1];
    const active = document.activeElement;

    if (event.shiftKey && (active === first || active === surfaceRef)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  };

  // 打开时把焦点移进浮层，否则键盘/读屏用户仍停在触发按钮上，
  // 拿不到"对话框已打开"的上下文（Drawer 同样处理）。
  // 关闭时交还给打开它的控件：浮层卸载后焦点会落到 <body>，键盘用户只能从
  // 页面开头重新 Tab。
  let wasOpen = false;
  createEffect(() => {
    if (open()) {
      wasOpen = true;
      const frame = requestAnimationFrame(() => {
        surfaceRef?.focus({ preventScroll: true });
      });
      onCleanup(() => cancelAnimationFrame(frame));
      return;
    }

    if (!wasOpen) return;
    wasOpen = false;
    trigger()?.focus({ preventScroll: true });
  });

  return (
    <Show when={show()}>
      <DialogContentContext.Provider value={{ setTitleID, setDescriptionID }}>
        <DialogPortal>
          <div
            role="presentation"
            class="fixed inset-0 select-none"
            aria-hidden="true"
          />
          <DialogOverlay
            dismissOnOverlayClick={local.dismissOnOverlayClick}
            class={local.overlayClass}
          />
          <span data-type="inside" class="sr-only" aria-hidden="true" />
          {/* biome-ignore lint/a11y/useAriaPropsSupportedByRole: role 为 dialog/alertdialog，二者都支持 aria-labelledby/aria-describedby */}
          <div
            ref={(el) => {
              surfaceRef = el;
            }}
            role={local.role ?? "dialog"}
            // 模态对话框要让辅助技术知道浮层之外的内容不可交互
            aria-modal="true"
            tabindex={-1}
            data-dialog-surface=""
            data-open={open()}
            class={local.class}
            classList={local.classList}
            aria-labelledby={titleID()}
            aria-describedby={descriptionID()}
            onKeyDown={trapFocus}
            onAnimationEnd={() => {
              if (open()) return;
              setShow(false);
            }}
            {...others}
          >
            {local.children}
          </div>
          <span data-type="inside" class="sr-only" aria-hidden="true" />
        </DialogPortal>
      </DialogContentContext.Provider>
    </Show>
  );
}

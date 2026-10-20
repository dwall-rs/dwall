import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  splitProps,
} from "solid-js";
import { useScrollLock } from "~/hooks/useScrollLock";
import type { DialogProps } from "./Dialog.types";
import { DialogContext } from "./Dialog.context";

export const Dialog = (props: DialogProps) => {
  const [local, others] = splitProps(props, [
    "open",
    "defaultOpen",
    "onOpenChange",
    "lockScroll",
    "dismissOnEscape",
    "children",
  ]);

  const initialOpen = local.open ?? local.defaultOpen ?? false;

  // 实际显示或卸载信号
  const [show, setShow] = createSignal(initialOpen);
  // 内部动画控制信号（非受控模式）
  const [internalOpen, setInternalOpen] = createSignal(
    local.defaultOpen ?? false,
  );
  // 实际动画控制状态：优先取受控 open，否则取内部状态
  const open = createMemo(() =>
    local.open === undefined ? internalOpen() : local.open,
  );
  const lockScroll = createMemo(() => props.lockScroll ?? true);

  // 打开期间默认锁定页面滚动：锁住文档滚动并拦截浮层之外的滚轮/触摸，
  // 内容区自身仍可滚动。传 lockScroll={false} 可关闭该行为。
  useScrollLock(() => open() && lockScroll(), {
    allowedSelector: "[data-dialog-surface]",
  });

  const setOpen = (next: boolean) => {
    if (local.open === undefined) {
      setInternalOpen(next);
    }
    local.onOpenChange?.(next);
  };

  // 打开期间按 Escape 关闭（与 Drawer 一致）
  createEffect(() => {
    if (!open() || local.dismissOnEscape === false) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented) return;
      event.preventDefault();
      setOpen(false);
    };
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  // 打开时立刻挂载；关闭时保留挂载直到动画结束，由 Content/Overlay 的
  // onAnimationEnd 负责卸载，保证退场动画能完整播放。
  createEffect(() => {
    if (open()) {
      setShow(true);
    }
  });

  return (
    <DialogContext.Provider
      value={{
        show,
        setShow,
        open,
        setOpen,
      }}
    >
      <div data-slot="dialog" {...others}>
        {local.children}
      </div>
    </DialogContext.Provider>
  );
};

import { Dialog } from "~/components/ui/dialog";
import type { AlertDialogProps } from "./AlertDialog.types";

/**
 * AlertDialog 复用 Dialog 的实现（受控/非受控状态机、Portal、Overlay、动画、
 * Title/Description 的 id 注入、滚动锁定），只在内容层保留自己的差异：
 * `role="alertdialog"`、无关闭按钮、点击外部不关闭、Escape 也不关闭
 * （alert dialog 要求用户做出显式选择，对齐 Base UI）。
 * 调用方仍可显式传 dismissOnEscape 覆盖。
 */
export const AlertDialog = (props: AlertDialogProps) => {
  return (
    <Dialog
      {...props}
      dismissOnEscape={props.dismissOnEscape ?? false}
      data-slot="alert-dialog"
    />
  );
};

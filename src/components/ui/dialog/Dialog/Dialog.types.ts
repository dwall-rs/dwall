import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseDialogProps {
  /**
   * 受控打开状态；不传则由 Dialog 内部自管理。
   * 传入后可以通过外部状态直接打开/关闭 Dialog。
   */
  open?: boolean;
  /** 非受控模式下的初始打开状态 */
  defaultOpen?: boolean;
  /** 打开状态变化时回调 */
  onOpenChange?: (open: boolean) => void;
  /**
   * 按 Escape 是否关闭，默认 true。
   * AlertDialog 需要"必须显式选择"，因此传 false（对齐 Base UI 的 alert dialog 语义）。
   */
  dismissOnEscape?: boolean;
  /**
   * 打开时是否锁定页面滚动，默认 true：锁住文档滚动并拦截浮层之外的
   * 滚轮/触摸滚动，内容区自身仍可滚动。
   */
  lockScroll?: boolean;
}

export type DialogProps = PolymorphicProps<
  "div",
  BaseProps & BaseDialogProps,
  false
>;

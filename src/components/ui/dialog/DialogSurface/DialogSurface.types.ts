import type { BaseProps, PolymorphicProps } from "~/types";

/**
 * DialogSurface 是 Dialog / AlertDialog 共用的「行为外壳」：
 * 负责 Portal、Overlay、隐藏哨兵节点、挂载/卸载动画，以及 Title/Description
 * 的 id 注入(通过 DialogContentContext)。视觉样式完全由调用方通过 class 提供，
 * 因此 DialogContent 与 AlertDialogContent 可以各自维护样式而不互相污染。
 */
export type DialogSurfaceProps = PolymorphicProps<
  "div",
  BaseProps & {
    /** 内容区 role，默认 "dialog"；AlertDialog 传 "alertdialog" */
    role?: "dialog" | "alertdialog";
    /** 点击 Overlay 是否关闭，默认 true；AlertDialog 传 false */
    dismissOnOverlayClick?: boolean;
    /** Overlay 的额外 class */
    overlayClass?: string;
  },
  false
>;

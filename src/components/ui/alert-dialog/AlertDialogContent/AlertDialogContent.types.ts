import type { DialogSurfaceProps } from "~/components/ui/dialog";
import type { BaseProps } from "~/types";

/**
 * AlertDialogContent 复用 DialogSurface 的浮层能力（Portal + Overlay + 定位），
 * 因此必须把 `DialogSurfaceProps` 一并带上：`overlayClass` /
 * `dismissOnOverlayClick` 在运行时本来就会透传，漏掉类型等于让调用方无法使用。
 */
export type AlertDialogContentProps = DialogSurfaceProps &
  BaseProps & {
    size?: "default" | "sm";
  };

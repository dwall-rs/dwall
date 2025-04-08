import Toast from "./Toast";
import { toast, showToast } from "./ToastService";
import type { ToastProps, ToastType, ToastPosition, ToastRef } from "./types";
import "./index.scss";

export type { ToastProps, ToastType, ToastPosition, ToastRef };
export { toast, showToast };
export default Toast;

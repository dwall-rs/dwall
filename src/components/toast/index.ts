/**
 * index.ts
 * Public API surface — only re-exports what consumers need.
 * Internal modules (hooks, utils, constants) are intentionally not re-exported.
 */

// ─── Core toast function ───────────────────────────────────────────────────────
export { toast } from "./state";

// ─── Components ───────────────────────────────────────────────────────────────
export { Toaster } from "./components/Toaster";
export { Toast } from "./components/Toast";

// ─── Hook for custom UIs ───────────────────────────────────────────────────────
export { useSonner } from "./hooks";

// ─── Types consumers need ──────────────────────────────────────────────────────
export type {
  Action,
  ExternalToast,
  ToasterProps,
  ToastClassnames,
  ToastT,
  ToastToDismiss,
} from "./types";

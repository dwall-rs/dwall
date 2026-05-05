import { createSignal, onCleanup, onMount } from "solid-js";
import { ToastState } from "./state";
import type { ToastT, ToastToDismiss } from "./types";

// ─── Document visibility ───────────────────────────────────────────────────────
/**
 * Returns a reactive boolean that tracks whether the browser tab is hidden.
 * Used to pause auto-dismiss timers while the user has switched tabs.
 */
export function useIsDocumentHidden() {
  const [isHidden, setIsHidden] = createSignal(document.hidden);

  onMount(() => {
    const handler = () => setIsHidden(document.hidden);
    document.addEventListener("visibilitychange", handler);
    onCleanup(() => document.removeEventListener("visibilitychange", handler));
  });

  return isHidden;
}

// ─── Active toasts (for external consumers) ────────────────────────────────────
/**
 * Subscribes to the global ToastState and returns the list of currently visible
 * toasts as a SolidJS signal. Useful for building custom toast UIs.
 */
export function useSonner() {
  const [activeToasts, setActiveToasts] = createSignal<ToastT[]>([]);

  onMount(() => {
    const unsubscribe = ToastState.subscribe((toast) => {
      if ((toast as ToastToDismiss).dismiss) {
        setActiveToasts((prev) => prev.filter((t) => t.id !== toast.id));
        return;
      }

      setActiveToasts((prev) => {
        const idx = prev.findIndex((t) => t.id === toast.id);
        if (idx !== -1) {
          return [
            ...prev.slice(0, idx),
            { ...prev[idx], ...(toast as ToastT) },
            ...prev.slice(idx + 1),
          ];
        }
        return [toast as ToastT, ...prev];
      });
    });

    onCleanup(unsubscribe);
  });

  return { toasts: activeToasts };
}

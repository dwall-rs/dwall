/**
 * hooks/useToastSubscription.ts
 * Subscribes to the global ToastState observable and maintains a local reactive
 * list of toasts for the Toaster component.
 * Single responsibility: state synchronization between the observer and Solid signals.
 */
import { createSignal, onCleanup, onMount } from "solid-js";
import { ToastState } from "../state";
import type { ToastT, ToastToDismiss } from "../types";

export function useToastSubscription() {
  const [toasts, setToasts] = createSignal<ToastT[]>([]);

  onMount(() => {
    const unsubscribe = ToastState.subscribe((incoming) => {
      if ((incoming as ToastToDismiss).dismiss) {
        // Mark as deleted — the Toast component will animate out
        setToasts((prev) =>
          prev.map((t) => (t.id === incoming.id ? { ...t, delete: true } : t)),
        );
        return;
      }

      setToasts((prev) => {
        const idx = prev.findIndex((t) => t.id === incoming.id);
        if (idx !== -1) {
          return [
            ...prev.slice(0, idx),
            { ...prev[idx], ...(incoming as ToastT) },
            ...prev.slice(idx + 1),
          ];
        }
        return [incoming as ToastT, ...prev];
      });
    });

    onCleanup(unsubscribe);
  });

  const removeToast = (toast: ToastT) => {
    if (!toast.delete) ToastState.dismiss(toast.id);
    setToasts((prev) => prev.filter((t) => t.id !== toast.id));
  };

  return { toasts, removeToast };
}

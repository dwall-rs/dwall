import { createEffect, createSignal, onCleanup } from "solid-js";
import type { HeightT, Position, ToastT } from "../types";

interface UseToastHeightOptions {
  toastRef: () => HTMLLIElement | undefined;
  toast: () => ToastT;
  mounted: () => boolean;
  setHeights: (updater: (prev: HeightT[]) => HeightT[]) => void;
}

export function useToastHeight(opts: UseToastHeightOptions) {
  const [initialHeight, setInitialHeight] = createSignal(0);

  const toast = opts.toast();

  // Register height on first mount
  createEffect(() => {
    const el = opts.toastRef();
    if (!el) return;
    const h = el.getBoundingClientRect().height;
    setInitialHeight(h);
    opts.setHeights((prev) => [
      {
        toastId: toast.id,
        height: h,
        position: toast.position as Position,
      },
      ...prev,
    ]);
    onCleanup(() =>
      opts.setHeights((prev) => prev.filter((x) => x.toastId !== toast.id)),
    );
  });

  // Keep height in sync when content changes
  createEffect(() => {
    if (!opts.mounted()) return;
    const el = opts.toastRef();
    if (!el) return;

    // Read deps so effect re-runs on content change
    void toast.title;
    void toast.description;
    void toast.jsx;
    void toast.action;
    void toast.cancel;

    const orig = el.style.height;
    el.style.height = "auto";
    const newH = el.getBoundingClientRect().height;
    el.style.height = orig;
    setInitialHeight(newH);

    opts.setHeights((prev) => {
      const exists = prev.find((x) => x.toastId === toast.id);
      if (!exists) {
        return [
          {
            toastId: toast.id,
            height: newH,
            position: toast.position as Position,
          },
          ...prev,
        ];
      }
      return prev.map((x) =>
        x.toastId === toast.id ? { ...x, height: newH } : x,
      );
    });
  });

  return { initialHeight };
}

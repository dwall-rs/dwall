import { type Accessor, createEffect, onCleanup } from "solid-js";

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled]):not([type='hidden'])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

export function useDialogFocus(open: Accessor<boolean>) {
  let previousFocus: HTMLElement = document.body;

  const focus = (element: HTMLElement) => {
    element.focus({ preventScroll: true });
  };

  const focusable = (surface: HTMLElement): HTMLElement[] =>
    [...surface.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)].filter(
      (element) =>
        !element.hidden && element.getAttribute("aria-hidden") !== "true",
    );

  const restoreFocus = () => {
    const target = previousFocus;
    previousFocus = document.body;
    if (target !== document.body && target.isConnected) {
      focus(target);
    }
  };

  createEffect(() => {
    if (!open()) return;

    previousFocus = (document.activeElement ?? document.body) as HTMLElement;
    onCleanup(restoreFocus);
  });

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Tab") return;

    const surface = event.currentTarget as HTMLElement;
    const items = focusable(surface);
    if (items.length === 0) {
      event.preventDefault();
      focus(surface);
      return;
    }

    const first = items[0];
    const last = items[items.length - 1];
    const active = document.activeElement;
    const outside = !surface.contains(active);

    if (event.shiftKey) {
      if (active === surface || active === first || outside) {
        event.preventDefault();
        focus(last);
      }
      return;
    }

    if (active === surface || active === last || outside) {
      event.preventDefault();
      focus(first);
    }
  };

  return { handleKeyDown };
}

/**
 * hooks/useToasterKeyboard.ts
 * Handles keyboard interactions for the Toaster:
 *   - Custom hotkey → expand the toast list and focus it
 *   - Escape → collapse the toast list
 * Single responsibility: keyboard event binding only.
 */
import { onCleanup, onMount } from "solid-js";

interface UseToasterKeyboardOptions {
  hotkey: string[];
  listRef: () => HTMLOListElement | undefined;
  setExpanded: (v: boolean) => void;
}

export function useToasterKeyboard(opts: UseToasterKeyboardOptions) {
  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      const hotkeyPressed =
        opts.hotkey.length > 0 &&
        opts.hotkey.every((k) => (e as any)[k] || e.code === k);

      if (hotkeyPressed) {
        opts.setExpanded(true);
        opts.listRef()?.focus();
      }

      if (
        e.code === "Escape" &&
        (document.activeElement === opts.listRef() ||
          opts.listRef()?.contains(document.activeElement))
      ) {
        opts.setExpanded(false);
      }
    };

    document.addEventListener("keydown", handler);
    onCleanup(() => document.removeEventListener("keydown", handler));
  });
}

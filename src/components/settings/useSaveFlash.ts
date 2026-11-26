// Responsibility: the "saved" tick next to a settings row — shown only after a write
// actually succeeded, and long enough to be seen.
import { createSignal, onCleanup } from "solid-js";

import { save } from "@/store/settings.store";

/** How long the tick stays visible after the file was written. */
const FLASH_MS = 1_200;

/**
 * Saves and flashes `key` only when the write succeeded.
 *
 * The previous per-component helper flashed at the start of the await and cleared it
 * when the await settled, so the tick really meant "saving" and a rejected write was
 * shown with the same success mark.
 */
export function useSaveFlash() {
  const [flash, setFlash] = createSignal<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const ok = async (key: string) => {
    if (!(await save())) return;

    setFlash(key);
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => setFlash(null), FLASH_MS);
  };

  onCleanup(() => {
    if (timer !== undefined) clearTimeout(timer);
  });

  return { flash, ok };
}

// Responsibility: fetch the current solar position for the configured position source, with an optional
// enable gate and polling. Keeps the IPC call out of layout components.
import { createResource, onCleanup, onMount } from "solid-js";

import { currentSolarPosition } from "@/ipc";
import { settingsStore } from "@/store/settings.store";

interface Options {
  /** Only fetch while this returns true (default: always). */
  enabled?: () => boolean;
  /** Re-fetch interval in ms; no polling when omitted. */
  pollMs?: number;
}

export function useSolarPosition(options: Options = {}) {
  const [position, { refetch }] = createResource(
    () =>
      (options.enabled?.() ?? true)
        ? settingsStore.config?.position_source
        : undefined,
    (ps) => currentSolarPosition(ps),
  );

  const pollMs = options.pollMs;
  if (pollMs) {
    onMount(() => {
      const timer = setInterval(() => refetch(), pollMs);
      onCleanup(() => clearInterval(timer));
    });
  }

  return { position, refetch };
}

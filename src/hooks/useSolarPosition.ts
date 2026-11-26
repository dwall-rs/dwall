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

type PositionSource = Parameters<typeof currentSolarPosition>[0];
type SolarPosition = Awaited<ReturnType<typeof currentSolarPosition>>;

/**
 * How long a reading is shared between callers.
 *
 * The stage and the engine panel each poll once a minute, so three windows of the same
 * reading overlapped and each invoke landed on a different blocking thread. The Rust
 * position cache is thread-local, so those calls did not share a sensor read either.
 * Sharing the result here makes the window cost one call per interval, while staying
 * well below the backend's 60 s maximum reading age.
 */
const POSITION_CACHE_TTL_MS = 30_000;

let cached: { key: string; at: number; value: SolarPosition } | undefined;
let inflight: { key: string; promise: Promise<SolarPosition> } | undefined;

/**
 * Loads a position, sharing a fresh reading and an in-flight request between callers.
 *
 * Errors are not cached: a denied location or a starting sensor must be retried by the
 * next poll rather than replayed for the TTL.
 */
export const loadSolarPosition = (
  source: PositionSource,
): Promise<SolarPosition> => {
  const key = JSON.stringify(source);
  const now = Date.now();

  if (cached?.key === key && now - cached.at < POSITION_CACHE_TTL_MS) {
    return Promise.resolve(cached.value);
  }

  if (inflight?.key === key) return inflight.promise;

  const promise = currentSolarPosition(source)
    .then((value) => {
      cached = { key, at: Date.now(), value };
      return value;
    })
    .finally(() => {
      if (inflight?.promise === promise) inflight = undefined;
    });

  inflight = { key, promise };
  return promise;
};

export function useSolarPosition(options: Options = {}) {
  const [position, { refetch }] = createResource(
    () =>
      (options.enabled?.() ?? true)
        ? settingsStore.config?.position_source
        : undefined,
    (ps) => loadSolarPosition(ps),
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

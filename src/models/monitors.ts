// Responsibility: pure mapping from the Rust monitor list to the UI monitor model. No store/i18n access.
import type { DisplayMonitor } from "@/ipc";
import type { Monitor } from "./types";

/** Pseudo-monitor whose settings apply to every monitor. */
export const ALL_MONITOR_ID = "all";

/** Build the UI monitor list, prepending the (already localized) "all monitors" entry. */
export const monitorList = (
  monitors: DisplayMonitor[],
  allLabel: string,
): Monitor[] => [
  { id: ALL_MONITOR_ID, name: allLabel },
  ...monitors.map((m) => ({ id: m.device_path, name: m.friendly_name })),
];

/** Look up a monitor (including the pseudo "all" entry) by id. */
export const monitorById = (
  monitors: DisplayMonitor[],
  allLabel: string,
  id: string,
): Monitor | undefined =>
  monitorList(monitors, allLabel).find((m) => m.id === id);

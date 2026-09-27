// Responsibility: adapt the Rust monitor list (catalog.store) to the UI's monitor model.
import { catalogStore } from "@/store/catalog.store";
import { t } from "@/i18n";
import type { Monitor } from "./types";

/** Pseudo monitor: applies one setting to all monitors. */
const ALL_MONITOR_ID = "all";

/** Monitor list (including the "all monitors" entry). */
export const monitorList = (): Monitor[] => [
  { id: ALL_MONITOR_ID, name: t("scope.allMonitors") },
  ...catalogStore.monitors.map((m) => ({
    id: m.device_path,
    name: m.friendly_name,
  })),
];

/** Look up a monitor by id. */
export const monitorById = (id: string): Monitor | undefined =>
  monitorList().find((m) => m.id === id);

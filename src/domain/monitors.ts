/* ===== src/domain/monitors.ts ===== */
// 职责：把 Rust 显示器列表（catalog.store）适配为 UI 的显示器模型。
import { catalogStore } from "@/store/catalog.store";
import { t } from "@/i18n";
import type { Monitor } from "./types";

/** 伪显示器：对所有显示器统一设置。 */
const ALL_MONITOR_ID = "all";

/** 显示器列表（含「所有显示器」项）。 */
export const monitorList = (): Monitor[] => [
  { id: ALL_MONITOR_ID, name: t("scope.allMonitors") },
  ...catalogStore.monitors.map((m) => ({
    id: m.device_path,
    name: m.friendly_name,
  })),
];

/** 按 id 取显示器。 */
export const monitorById = (id: string): Monitor | undefined =>
  monitorList().find((m) => m.id === id);

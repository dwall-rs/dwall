// Responsibility: read-only view-model selectors over the Rust catalog — theme/monitor lists plus the
// localized "all monitors" pseudo-entry. Keeps models/ pure by owning the store + i18n access here.
import { t } from "@/i18n";

import {
  monitorById as findMonitor,
  monitorList as buildMonitorList,
} from "@/models/monitors";
import {
  themeById as findTheme,
  themeList as buildThemeList,
} from "@/models/themes";

import { catalogStore } from "./catalog.store";

const allMonitorsLabel = (): string => t("scope.allMonitors");

export const monitorList = () =>
  buildMonitorList(catalogStore.monitors, allMonitorsLabel());

export const monitorById = (id: string) =>
  findMonitor(catalogStore.monitors, allMonitorsLabel(), id);

export const themeList = () => buildThemeList(catalogStore.themes);

export const themeById = (id: string | undefined) =>
  findTheme(catalogStore.themes, id);

/**
 * Themes whose frames are on disk, so they can actually be applied.
 *
 * Everything that selects a theme works from this list: an uninstalled theme can be
 * downloaded from the library, but it cannot be assigned or pooled before that.
 */
export const availableThemes = () =>
  themeList().filter((theme) => !theme.needsDownload);

/** Ids of the themes that can be applied. */
export const availableThemeIds = () =>
  availableThemes().map((theme) => theme.id);

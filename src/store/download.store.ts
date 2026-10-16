// Responsibility: theme download state — start, cancel and report one download at a time. Adapter over the IPC layer; the Config comes from settings.store.
import { createStore } from "solid-js/store";

import {
  cancelThemeDownload,
  downloadThemeAndExtract,
  onThemeDownloadProgress,
} from "@/ipc";
import { downloadPercent, isCancellation } from "@/models/download";
import { settingsStore } from "./settings.store";

interface DownloadState {
  /** Theme downloading now, or the theme whose download failed. */
  themeId: string | null;
  status: "idle" | "downloading" | "failed";
  /** Whole percent, or null while the size is unknown. */
  percent: number | null;
  error: string | null;
}

const [downloadStore, setDownloadStore] = createStore<DownloadState>({
  themeId: null,
  status: "idle",
  percent: null,
  error: null,
});

export { downloadStore };

/** Whether `themeId` is the download in flight. */
export const isDownloading = (themeId: string) =>
  downloadStore.status === "downloading" && downloadStore.themeId === themeId;

/** Failure of `themeId`'s download, if that is the one that failed. */
export const downloadError = (themeId: string) =>
  downloadStore.status === "failed" && downloadStore.themeId === themeId
    ? downloadStore.error
    : null;

/**
 * Downloads and installs `themeId`, resolving true once its frames are on disk.
 *
 * Progress arrives as an event carrying the theme id, so a stray message cannot
 * move another download's bar.
 */
export const downloadTheme = async (themeId: string): Promise<boolean> => {
  const config = settingsStore.config;
  if (!config || downloadStore.status === "downloading") return false;

  setDownloadStore({
    themeId,
    status: "downloading",
    percent: null,
    error: null,
  });

  const unlisten = await onThemeDownloadProgress((progress) => {
    if (progress.taskId !== themeId) return;
    setDownloadStore("percent", downloadPercent(progress));
  });

  try {
    await downloadThemeAndExtract(config, themeId);
    setDownloadStore({
      themeId: null,
      status: "idle",
      percent: null,
      error: null,
    });
    return true;
  } catch (error) {
    // Cancelling is a user decision, not a failure: return to the idle button.
    const cancelled = isCancellation(error);
    setDownloadStore({
      themeId: cancelled ? null : themeId,
      status: cancelled ? "idle" : "failed",
      percent: null,
      error: cancelled ? null : String(error),
    });
    return false;
  } finally {
    unlisten();
  }
};

/** Asks the backend to cancel the download in flight. */
export const cancelDownload = async () => {
  const themeId = downloadStore.themeId;
  if (downloadStore.status === "downloading" && themeId) {
    await cancelThemeDownload(themeId);
  }
};

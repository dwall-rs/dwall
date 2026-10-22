// Responsibility: theme download state — start, cancel and report one download at a time. Adapter over the IPC layer; the Config comes from settings.store.
import { createStore } from "solid-js/store";

import {
  cancelThemeDownload,
  downloadThemeAndExtract,
  onThemeDownloadProgress,
} from "@/ipc";
import { downloadPercent, isCancellation } from "@/models/download";
import { t } from "@/i18n";
import { toast } from "@/components/ui/toast";
import { logger } from "@/utils";
import { markInstalled } from "./catalog.store";
import { settingsStore } from "./settings.store";

const log = logger.child("download");

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

  // Registered inside the `try`: if listening fails there is no progress to show,
  // and the store must not stay "downloading" forever with every button disabled.
  let unlisten: (() => void) | undefined;

  try {
    unlisten = await onThemeDownloadProgress((progress) => {
      if (progress.taskId !== themeId) return;
      setDownloadStore("percent", downloadPercent(progress));
    });

    await downloadThemeAndExtract(config, themeId);
    // Tells every view that reads frames from disk that this theme has them now.
    markInstalled(themeId);
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
    const message = cancelled ? null : String(error);
    if (message) {
      // The library shows a compact button with no room for a message, so the failure
      // is reported here; the stage's install prompt repeats it next to the retry.
      toast.error(t("stage.downloadFailed"), { description: message });
    }
    setDownloadStore({
      themeId: cancelled ? null : themeId,
      status: cancelled ? "idle" : "failed",
      percent: null,
      error: message,
    });
    return false;
  } finally {
    unlisten?.();
  }
};

/** Asks the backend to cancel the download in flight. */
export const cancelDownload = async () => {
  const themeId = downloadStore.themeId;
  if (downloadStore.status === "downloading" && themeId) {
    try {
      await cancelThemeDownload(themeId);
    } catch (e) {
      // The download may already be over, or the command may be unreachable; the
      // progress event (or the failure path above) settles the state either way.
      log.warn("Failed to cancel the download", e);
    }
  }
};

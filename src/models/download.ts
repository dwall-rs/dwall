// Responsibility: pure mapping of download progress and errors into what the UI shows. No store/i18n access.
import type { ThemeDownloadProgress } from "@/ipc";

/**
 * Progress in whole percent, or null when the server reported no content length.
 *
 * GitHub sends `Content-Length` for release assets, but a mirror or a proxy may
 * not, and the UI has to show *something* rather than an empty bar.
 */
export const downloadPercent = (
  progress: Pick<ThemeDownloadProgress, "downloadedBytes" | "totalBytes">,
): number | null => {
  if (progress.totalBytes <= 0) return null;

  const percent = Math.round(
    (progress.downloadedBytes / progress.totalBytes) * 100,
  );
  return Math.max(0, Math.min(100, percent));
};

/** Whether the error is the user cancelling the download rather than a failure. */
export const isCancellation = (error: unknown) =>
  String(error).includes("Download cancelled");

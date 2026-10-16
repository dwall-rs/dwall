// Responsibility: install affordance for a theme that is not on disk yet — start the download, show its progress, cancel it, or report a failure and retry.
import { Show } from "solid-js";

import { Download, X } from "lucide-solid";

import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import {
  cancelDownload,
  downloadError,
  downloadStore,
  downloadTheme,
  isDownloading,
} from "@/store/download.store";

interface Props {
  themeId: string;
  /** Called after a successful download, so the caller reloads the theme's frames. */
  onInstalled: () => void;
}

export function ThemeDownload(props: Props) {
  const start = async () => {
    if (await downloadTheme(props.themeId)) props.onInstalled();
  };

  const width = () =>
    downloadStore.percent === null ? "100%" : `${downloadStore.percent}%`;

  return (
    <div class="flex flex-col items-center gap-2.5">
      <Show
        when={isDownloading(props.themeId)}
        fallback={
          <Button onClick={() => void start()} class="px-6 py-2">
            <Download class="size-3.5" />
            {t("stage.download")}
          </Button>
        }
      >
        <div class="h-1.5 w-56 overflow-hidden rounded-full bg-border">
          <div
            class={clsx(
              "h-full rounded-full bg-primary transition-all duration-200",
              downloadStore.percent === null && "animate-pulse",
            )}
            style={{ width: width() }}
          />
        </div>
        <span class="font-mono text-[11px] text-muted-foreground">
          {downloadStore.percent === null
            ? t("stage.downloading")
            : `${t("stage.downloading")} ${downloadStore.percent}%`}
        </span>
        <Button variant="destructive" onClick={() => void cancelDownload()}>
          <X class="size-3.5" />
          {t("stage.cancelDownload")}
        </Button>
      </Show>

      <Show when={downloadError(props.themeId)}>
        {(error) => (
          <p class="max-w-90 text-center font-mono text-[11px] leading-relaxed text-destructive">
            {t("stage.downloadFailed")} {error()}
          </p>
        )}
      </Show>
    </div>
  );
}

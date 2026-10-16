// Responsibility: compact install control for one theme that is not on disk — start the download, show its percent, cancel it, retry after a failure. Used wherever a theme is listed (library tile, random pool collage), so both modes install themes the same way.
import { Show } from "solid-js";

import { Download, X } from "lucide-solid";

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
  class?: string;
}

export function ThemeDownloadButton(props: Props) {
  /** One download runs at a time, so the other buttons wait for it. */
  const busy = () => downloadStore.status === "downloading";

  /** The failure text doubles as the tooltip: the button itself has no room for it. */
  const failure = () => downloadError(props.themeId);

  const start = (event: MouseEvent) => {
    // The tile underneath selects the theme; installing it is a separate action.
    event.stopPropagation();
    void downloadTheme(props.themeId);
  };

  const cancel = (event: MouseEvent) => {
    event.stopPropagation();
    void cancelDownload();
  };

  return (
    <Show
      when={isDownloading(props.themeId)}
      fallback={
        <button
          type="button"
          title={failure() ?? t("stage.download")}
          aria-label={t("stage.download")}
          disabled={busy()}
          onClick={start}
          class={clsx(
            "grid size-6 cursor-pointer place-items-center rounded-md border bg-background/85 shadow-[0_2px_8px_rgba(0,0,0,.24)] backdrop-blur-md transition-colors disabled:cursor-default disabled:opacity-40",
            failure()
              ? "border-destructive/50 text-destructive hover:bg-destructive/10"
              : "border-border text-foreground hover:bg-accent",
            props.class,
          )}
        >
          <Download class="size-3.5" />
        </button>
      }
    >
      <button
        type="button"
        title={t("stage.cancelDownload")}
        aria-label={t("stage.cancelDownload")}
        onClick={cancel}
        class={clsx(
          "flex h-6 cursor-pointer items-center gap-1 rounded-md border border-border bg-background/85 px-1.5 font-mono text-[10px] text-foreground shadow-[0_2px_8px_rgba(0,0,0,.24)] backdrop-blur-md transition-colors hover:bg-accent",
          props.class,
        )}
      >
        {downloadStore.percent === null
          ? t("stage.downloading")
          : `${downloadStore.percent}%`}
        <X class="size-3" />
      </button>
    </Show>
  );
}

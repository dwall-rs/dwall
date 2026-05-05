import { createEffect, createSignal, onMount, Show } from "solid-js";

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { message } from "@tauri-apps/plugin-dialog";

import { Button } from "~/components/button";
import { Progress } from "~/components/progress";

import { downloadThemeAndExtract, cancelThemeDownload } from "~/commands";

import { useConfig } from "~/contexts";
import { t } from "~/i18n";
import { toast } from "~/components/toast";

interface DownloadProgress {
  theme_id: string;
  downloaded_bytes: number;
  total_bytes: number;
}

const window = getCurrentWebviewWindow();

interface DownloadProps {
  id: string;
  onFinished: () => void;
}

const Download = (props: DownloadProps) => {
  const { data: config } = useConfig();
  const [percent, setPercent] = createSignal<number>();
  const [isCancelling, setIsCancelling] = createSignal(false);
  const [warning, setWarning] = createSignal<string>();

  const handleCancelDownload = async () => {
    if (isCancelling()) return;

    setIsCancelling(true);
    try {
      await cancelThemeDownload(props.id);
      // Cancellation request sent, but actual cancellation will be handled by backend
    } catch (e) {
      message(t("theme.message.downloadFailed", { error: String(e) }), {
        title: t("theme.title.downloadFailed"),
        kind: "error",
      });
    }
  };

  onMount(async () => {
    const unlisten = await window.listen<DownloadProgress>(
      "download-theme",
      (e) => {
        const { total_bytes, downloaded_bytes } = e.payload;
        if (total_bytes === 0) {
          setWarning(t("theme.message.fileSizeWarning"));
          setPercent(100);
        } else {
          setPercent(Math.round((downloaded_bytes / total_bytes) * 1000) / 10);
        }
      },
    );

    try {
      await downloadThemeAndExtract(config()!, props.id);
    } catch (e) {
      // Check if the error is caused by download cancellation
      if (String(e).includes("Download cancelled")) {
        toast.success(t("theme.message.downloadCancelled"));
      } else {
        message(t("theme.message.downloadFailed", { error: String(e) }), {
          title: t("theme.title.downloadFailed"),
          kind: "error",
        });
      }
    } finally {
      props.onFinished();
      setPercent();
      unlisten();
    }
  });

  createEffect(
    () =>
      warning() &&
      toast.warning(warning(), {
        duration: 10000,
      }),
  );

  return (
    <>
      <Progress
        value={percent() ?? 0}
        class="w-full max-w-sm absolute bottom-10 left-1/2 -translate-x-1/2"
      />
      <Show when={!isCancelling()}>
        <Button variant="destructive" onClick={handleCancelDownload}>
          {t("theme.button.cancel") || "Cancel"}
        </Button>
      </Show>
    </>
  );
};

export default Download;

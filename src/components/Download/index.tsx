import { createEffect, createSignal, onMount, Show } from "solid-js";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { message } from "@tauri-apps/plugin-dialog";
import { LazyButton, LazyProgress } from "~/lazy";
import { useToast } from "~/components//Toast";
import { downloadThemeAndExtract, cancelThemeDownload } from "~/commands";
import { useConfig, useTheme, useTranslations } from "~/contexts";
import "./index.scss";

interface DownloadProgress {
  theme_id: string;
  downloaded_bytes: number;
  total_bytes: number;
}

const window = getCurrentWebviewWindow();

const Download = () => {
  const { translate, translateErrorMessage } = useTranslations();
  const theme = useTheme();
  const { data: config } = useConfig();
  const [percent, setPercent] = createSignal<number>();
  const [isCancelling, setIsCancelling] = createSignal(false);
  const [warning, setWarning] = createSignal<string>();
  const toast = useToast();

  const onFinished = () => {
    theme.setDownloadThemeID();
    theme.handleThemeSelection(theme.menuItemIndex()!);
  };

  const handleCancelDownload = async () => {
    if (isCancelling()) return;

    setIsCancelling(true);
    try {
      await cancelThemeDownload(theme.downloadThemeID()!);
      // Cancellation request sent, but actual cancellation will be handled by backend
    } catch (e) {
      message(translateErrorMessage("message-download-faild", e), {
        title: translate("title-download-faild"),
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
          setWarning(translate("message-file-size-warning"));
          setPercent(100);
        } else {
          setPercent(Math.round((downloaded_bytes / total_bytes) * 1000) / 10);
        }
      },
    );

    try {
      await downloadThemeAndExtract(config()!, theme.downloadThemeID()!);
    } catch (e) {
      // Check if the error is caused by download cancellation
      if (String(e).includes("Download cancelled")) {
        toast.success(translate("message-download-cancelled"), {
          closable: false,
        });
      } else {
        message(translateErrorMessage("message-download-faild", e), {
          title: translate("title-download-faild"),
          kind: "error",
        });
      }
    } finally {
      onFinished();
      setPercent();
      unlisten();
    }
  });

  createEffect(
    () =>
      warning() &&
      toast.warning(warning(), {
        duration: 10000,
        maxWidth: 480,
      }),
  );

  return (
    <div class="download-container">
      <LazyProgress class="download-progress" value={percent()} />
      <Show when={!isCancelling()}>
        <LazyButton onClick={handleCancelDownload} appearance="danger">
          {translate("button-cancel") || "Cancel"}
        </LazyButton>
      </Show>
    </div>
  );
};

export default Download;

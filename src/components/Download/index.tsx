import { createSignal, onMount, Show } from "solid-js";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { message } from "@tauri-apps/plugin-dialog";
import { LazyButton, LazyProgress } from "~/lazy";
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
  const { translate } = useTranslations();
  const theme = useTheme();
  const { data: config } = useConfig();
  const [percent, setPercent] = createSignal<number>();
  const [isCancelling, setIsCancelling] = createSignal(false);

  const onFinished = () => {
    theme.setDownloadThemeID();
    theme.handleThemeSelection(theme.menuItemIndex()!);
  };

  const handleCancelDownload = async () => {
    if (isCancelling()) return;

    setIsCancelling(true);
    try {
      await cancelThemeDownload(theme.downloadThemeID()!);
      // 取消操作已发送，但实际取消会在后端处理
    } catch (e) {
      message(
        translate("title-download-faild", {
          error: String(e),
        }),
        {
          title: translate("title-download-faild"),
          kind: "error",
        },
      );
    }
  };

  onMount(async () => {
    const unlisten = await window.listen<DownloadProgress>(
      "download-theme",
      (e) => {
        const { total_bytes, downloaded_bytes } = e.payload;
        setPercent(Math.round((downloaded_bytes / total_bytes) * 1000) / 10);
      },
    );

    try {
      await downloadThemeAndExtract(config()!, theme.downloadThemeID()!);
    } catch (e) {
      // 检查是否是取消下载导致的错误
      if (String(e).includes("Download cancelled")) {
        // message(
        //   translate("download-cancelled", {
        //     themeId: theme.downloadThemeID()!,
        //   }) || `Download of ${theme.downloadThemeID()} was cancelled`,
        //   {
        //     title:
        //       translate("download-cancelled-title") || "Download Cancelled",
        //     kind: "info",
        //   },
        // );
      } else {
        message(
          translate("title-download-faild", {
            error: String(e),
          }),
          {
            title: translate("title-download-faild"),
            kind: "error",
          },
        );
      }
    } finally {
      onFinished();
      setPercent();
      unlisten();
    }
  });

  return (
    <div class="download-container">
      <LazyProgress class="download-progress" value={percent()} />
      <Show when={!isCancelling()}>
        <LazyButton class="cancel-download-btn" onClick={handleCancelDownload}>
          {translate("button-cancel") || "Cancel"}
        </LazyButton>
      </Show>
    </div>
  );
};

export default Download;

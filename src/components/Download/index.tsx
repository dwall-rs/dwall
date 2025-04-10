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
      // 取消操作已发送，但实际取消会在后端处理
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
        if (total_bytes === 0 && downloaded_bytes > 0) {
          setWarning(
            "因无法获取文件大小导致无法计算下载进度，请更换支持转发响应头的 Github 镜像模板",
          );
        }
        setPercent(Math.round((downloaded_bytes / total_bytes) * 1000) / 10);
      },
    );

    try {
      await downloadThemeAndExtract(config()!, theme.downloadThemeID()!);
    } catch (e) {
      // 检查是否是取消下载导致的错误
      if (String(e).includes("Download cancelled")) {
        toast.success("下载已取消", {
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

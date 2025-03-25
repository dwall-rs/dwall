import { createSignal, onMount } from "solid-js";
import { LazyProgress } from "~/lazy";
import "./index.scss";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { downloadThemeAndExtract } from "~/commands";
import { useAppContext } from "~/context";
import { message } from "@tauri-apps/plugin-dialog";
import { useTranslations } from "../TranslationsContext";

interface DownloadProgress {
  theme_id: string;
  downloaded_bytes: number;
  total_bytes: number;
}

const window = getCurrentWebviewWindow();

const Download = () => {
  const { translate } = useTranslations();
  const { config, theme } = useAppContext()!;
  const [percent, setPercent] = createSignal<number>();

  const onFinished = () => {
    theme.setDownloadThemeID();
    theme.handleThemeSelection(theme.menuItemIndex()!);
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
      message(
        translate("title-download-faild", {
          error: String(e),
        }),
        {
          title: translate("title-download-faild"),
          kind: "error",
        },
      );
    } finally {
      onFinished();
      setPercent();
      unlisten();
    }
  });

  return <LazyProgress class="download-progress" value={percent()} />;
};

export default Download;

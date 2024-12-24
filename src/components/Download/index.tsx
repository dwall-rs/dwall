import { createSignal, onMount } from "solid-js";
import { LazyProgress } from "~/lazy";
import "./index.scss";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { downloadThemeAndExtract } from "~/commands";
import { useAppContext } from "~/context";
import { message } from "@tauri-apps/plugin-dialog";
import { translate } from "~/utils/i18n";

interface DownloadProgress {
  theme_id: string;
  downloaded_bytes: number;
  total_bytes: number;
}

interface DownloadProps {
  themeID: string;
  onFinished: () => void;
}

const window = getCurrentWebviewWindow();

const Download = (props: DownloadProps) => {
  const { config, translations } = useAppContext()!;
  const [percent, setPercent] = createSignal<number>();

  onMount(async () => {
    const unlisten = await window.listen<DownloadProgress>(
      "download-theme",
      (e) => {
        const { total_bytes, downloaded_bytes } = e.payload;
        setPercent(Math.round((downloaded_bytes / total_bytes) * 1000) / 10);
      },
    );

    try {
      await downloadThemeAndExtract(config()!, props.themeID);
    } catch (e) {
      message(
        translate(translations()!, "title-download-faild", {
          error: String(e),
        }),
        {
          title: translate(translations()!, "title-download-faild"),
          kind: "error",
        },
      );
    } finally {
      props.onFinished();
      setPercent();
      unlisten();
    }
  });

  return <LazyProgress class="download-progress" value={percent()} />;
};

export default Download;

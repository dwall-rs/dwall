import { createSignal, onMount } from "solid-js";
import { LazyProgress } from "~/lazy";
import "./index.scss";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { downloadThemeAndExtract } from "~/commands";
import { useAppContext } from "~/context";
import { message } from "@tauri-apps/plugin-dialog";

interface ProgressPayload {
  id: string;
  progress: number;
  total: number;
}

interface DownloadProps {
  themeID: string;
  onFinished: () => void;
}

const window = getCurrentWebviewWindow();

const Download = (props: DownloadProps) => {
  const { config } = useAppContext()!;
  const [percent, setPercent] = createSignal<number>();

  onMount(async () => {
    const unlisten = await window.listen<ProgressPayload>(
      "download-theme",
      (e) => {
        const { total, progress } = e.payload;
        setPercent(Math.round((progress / total) * 1000) / 10);
      },
    );

    try {
      await downloadThemeAndExtract(config()!, props.themeID);
    } catch (e) {
      message(`${e}\n\n具体错误请查看日志：dwall_settings_lib.log`, {
        title: "下载失败",
        kind: "error",
      });
    } finally {
      props.onFinished();
      setPercent();
      unlisten();
    }
  });

  return <LazyProgress class="download-progress" value={percent()} />;
};

export default Download;

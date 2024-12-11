import { createSignal, onMount } from "solid-js";
import { LazyProgress } from "~/lazy";
import "./index.scss";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { downloadThemeAndExtract } from "~/commands";
import { useAppContext } from "~/context";

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

    await downloadThemeAndExtract(config()!, props.themeID);

    props.onFinished();
    setPercent();

    unlisten();
  });

  return <LazyProgress class="download-progress" value={percent()} />;
};

export default Download;

import { relaunch } from "@tauri-apps/plugin-process";
import type { Update } from "@tauri-apps/plugin-updater";
import { createSignal, onMount } from "solid-js";
import { killDaemon } from "~/commands";
import { LazyDialog, LazyProgress } from "~/lazy";

interface UpdateDialogProps {
  update: Update;
}

const UpdateDialog = (props: UpdateDialogProps) => {
  const [total, setTotal] = createSignal<number | undefined>();
  const [downloaded, setDownloaded] = createSignal<number | undefined>();

  onMount(async () => {
    await killDaemon();
    await props.update.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          setTotal(event.data.contentLength ?? 0);
          break;
        case "Progress":
          setDownloaded((prev) => (prev ?? 0) + event.data.chunkLength);
          break;
        case "Finished":
          break;
      }
    });

    await relaunch();
  });

  return (
    <LazyDialog
      show={!!props.update}
      onClose={() => {}}
      maskClosable={false}
      size="large"
    >
      <LazyProgress
        style={{ width: "540px" }}
        thickness="large"
        max={total()}
        value={downloaded()}
      />
    </LazyDialog>
  );
};

export default UpdateDialog;

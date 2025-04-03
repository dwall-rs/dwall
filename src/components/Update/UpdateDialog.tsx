import { relaunch } from "@tauri-apps/plugin-process";
import type { Update } from "@tauri-apps/plugin-updater";
import { createSignal, onMount } from "solid-js";
import { LazyProgress } from "~/lazy";
import Dialog from "../Dialog";

import { useTranslations, useUpdate } from "~/contexts";
import { message } from "@tauri-apps/plugin-dialog";

interface UpdateDialogProps {
  update: Update;
}

const UpdateDialog = (props: UpdateDialogProps) => {
  const { translate, translateErrorMessage } = useTranslations();
  const { setShowUpdateDialog } = useUpdate();

  const [total, setTotal] = createSignal<number | undefined>();
  const [downloaded, setDownloaded] = createSignal<number | undefined>();

  onMount(async () => {
    try {
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
    } catch (error) {
      await message(translateErrorMessage("message-update-failed", error), {
        kind: "error",
      });
      setShowUpdateDialog(false);
    }
  });

  return (
    <Dialog
      open={!!props.update}
      maskClosable={false}
      title={translate("title-downloading-new-version")}
    >
      <LazyProgress
        style={{ width: "480px" }}
        thickness="large"
        max={total()}
        value={downloaded()}
      />
    </Dialog>
  );
};

export default UpdateDialog;

import { createEffect, createSignal, onMount } from "solid-js";
import { AiOutlineDownload } from "solid-icons/ai";

import type { Update } from "@tauri-apps/plugin-updater";
import { message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";

import { LazyButton, LazyProgress } from "~/lazy";
import Dialog from "../Dialog";

import { useToast, useTranslations, useUpdate } from "~/contexts";

interface UpdateDialogProps {
  update: Update;
}

const UpdateDialog = (props: UpdateDialogProps) => {
  const toast = useToast();
  const { translate, translateErrorMessage } = useTranslations();
  const { setShowUpdateDialog } = useUpdate();

  const [total, setTotal] = createSignal<number | undefined>();
  const [downloaded, setDownloaded] = createSignal<number | undefined>();
  const [error, setError] = createSignal<string | undefined>();

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
    } catch (error) {
      const errorMessage = translateErrorMessage(
        "message-update-failed",
        error,
      );
      await message(errorMessage, {
        kind: "error",
      });
      setError(errorMessage);
      setShowUpdateDialog(false);
    }
  });

  const updateErrorHelpMessage = (message: string) => {
    return (
      <div>
        <h4>{message}</h4>
        <div>
          {translate("help-update-failed")}
          <LazyButton
            onClick={() =>
              open(
                (
                  props.update.rawJson.platforms as Record<
                    string,
                    Record<string, string>
                  >
                )["windows-x86_64"].url,
              )
            }
            icon={<AiOutlineDownload />}
            appearance="primary"
            size="small"
          />
        </div>
      </div>
    );
  };

  createEffect(() => {
    error() &&
      toast.error(updateErrorHelpMessage(error()!), {
        position: "bottom-right",
        duration: 5000,
      });
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

import { createResource, createSignal } from "solid-js";
import { message } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import { useTranslations } from "~/contexts";

export const useUpdateManager = () => {
  const { translateErrorMessage } = useTranslations();

  const [showUpdateDialog, setShowUpdateDialog] = createSignal<boolean>();
  const [update, { refetch: recheckUpdate }] = createResource(async () => {
    try {
      return await check();
    } catch (e) {
      console.error(e);
      message(translateErrorMessage("message-update-failed", e), {
        kind: "error",
      });
      return null;
    }
  });

  return {
    showUpdateDialog,
    setShowUpdateDialog,
    update,
    recheckUpdate,
  };
};

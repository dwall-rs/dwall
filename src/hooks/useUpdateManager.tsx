import { createResource } from "solid-js";
import { message } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import { t } from "~/i18n";

export const useUpdateManager = () => {
  const [update, { refetch: recheckUpdate }] = createResource(async () => {
    try {
      return await check();
    } catch (e) {
      console.error(e);
      message(t("update.message.updateFailed", { error: String(e) }), {
        kind: "error",
      });
      return;
    }
  });

  return {
    update,
    recheckUpdate,
  };
};

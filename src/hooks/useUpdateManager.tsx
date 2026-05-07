import { createResource } from "solid-js";
import { message } from "@tauri-apps/plugin-dialog";
import { t } from "~/i18n";
import { useConfig } from "~/contexts";
import { checkForUpdates } from "~/commands";

export const useUpdateManager = () => {
  const { data: config } = useConfig();

  const [update, { refetch: recheckUpdate }] = createResource(
    config,
    async (c) => {
      try {
        return await checkForUpdates(c?.network);
      } catch (e) {
        console.error(e);
        message(t("update.message.updateFailed", { error: String(e) }), {
          kind: "error",
        });
        return;
      }
    },
  );

  return {
    update,
    recheckUpdate,
  };
};

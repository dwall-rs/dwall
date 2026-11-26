// Responsibility: run a "open this in Explorer/browser" action and report a rejection
// instead of leaving an unhandled promise rejection.
import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { logger } from "@/utils";

const log = logger.child("external");

/**
 * Runs a backend action that leaves the app (opening a URL, a directory, the log folder).
 *
 * These are fire-and-forget clicks, so every caller used to write `void openUrl(...)`:
 * when the shell refused (no browser handler, a removed directory, a denied open), the
 * promise rejected with nothing observing it and the button appeared inert.
 */
export const runExternalAction = async (
  description: string,
  action: () => Promise<void>,
): Promise<void> => {
  try {
    await action();
  } catch (e) {
    log.error(`Failed to ${description}`, e);
    toast.error(t("common.operationFailed"), { description: String(e) });
  }
};

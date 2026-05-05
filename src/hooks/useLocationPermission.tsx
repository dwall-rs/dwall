import {
  openPrivacyLocationSettings,
  requestLocationPermission,
} from "~/commands";

import { ask } from "@tauri-apps/plugin-dialog";
import { t } from "~/i18n";

/**
 * Location permission management Hook, used to handle location permission requests and related operations
 * @param mutate Configuration update function
 * @returns Location permission related methods
 */
export const useLocationPermission = (
  mutate: (fn: (prev: Config) => Config) => void,
) => {
  // Check location permission
  const checkLocationPermission = async (): Promise<boolean> => {
    try {
      await requestLocationPermission();
      return true;
    } catch (e) {
      console.error(e);
      const shouldContinue = await ask(t("common.message.locationPermission"), {
        kind: "warning",
      });

      if (!shouldContinue) {
        await openPrivacyLocationSettings();
        return false;
      }

      mutate((prev) => ({
        ...prev!,
        position_source: { type: "MANUAL" },
      }));
      return false;
    }
  };

  return { checkLocationPermission };
};

import {
  openPrivacyLocationSettings,
  requestLocationPermission,
} from "~/commands";

import { ask } from "@tauri-apps/plugin-dialog";

import { useTranslations } from "~/contexts";

/**
 * Location permission management Hook, used to handle location permission requests and related operations
 * @param mutate Configuration update function
 * @param setShowSettings Function to set the display of settings panel
 * @returns Location permission related methods
 */
export const useLocationPermission = (
  mutate: (fn: (prev: Config) => Config) => void,
  setShowSettings: (show: boolean) => void,
  setMenuItemIndex: Setter<number | undefined>,
) => {
  const { translate } = useTranslations();
  // Check location permission
  const checkLocationPermission = async (): Promise<boolean> => {
    try {
      await requestLocationPermission();
      return true;
    } catch (e) {
      console.error(e);
      const shouldContinue = await ask(
        translate("message-location-permission"),
        { kind: "warning" },
      );

      if (!shouldContinue) {
        await openPrivacyLocationSettings();
        return false;
      }

      mutate((prev) => ({
        ...prev!,
        position_source: { type: "MANUAL" },
      }));
      setMenuItemIndex();
      setShowSettings(true);
      return false;
    }
  };

  return { checkLocationPermission };
};

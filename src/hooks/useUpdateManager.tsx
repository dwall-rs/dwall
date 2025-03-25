import { createSignal } from "solid-js";

/**
 * Update Manager Hook for handling application update related functionalities
 * @param update Update resource
 * @param recheckUpdate Function to recheck for updates
 * @returns Update manager related states and methods
 */
export const useUpdateManager = (
  update: Resource<Update | null>,
  recheckUpdate: () => void,
) => {
  const [showUpdateDialog, setShowUpdateDialog] = createSignal<boolean>();

  return {
    showUpdateDialog,
    setShowUpdateDialog,
    update,
    recheckUpdate,
  };
};

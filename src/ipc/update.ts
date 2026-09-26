import { invoke } from "@tauri-apps/api/core";
import { Update } from "@tauri-apps/plugin-updater";

import type { Network, UpdateMetadata } from "./types";

export type { Update } from "@tauri-apps/plugin-updater";

export const checkForUpdates = async (network?: Network) => {
  const metadata = await invoke<UpdateMetadata | null>(
    "check_for_updates_cmd",
    {
      network,
    },
  );
  return metadata
    ? new Update({ ...metadata, body: metadata.body ?? undefined, rawJson: {} })
    : null;
};

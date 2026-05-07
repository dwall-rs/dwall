import { invoke } from "@tauri-apps/api/core";
import { Update } from "@tauri-apps/plugin-updater";

interface UpdateMetadata {
  rid: number;
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
  rawJson: Record<string, unknown>;
}

export const checkForUpdates = async (network?: Network) => {
  const metadata = await invoke<UpdateMetadata>("check_for_updates_cmd", {
    network,
  });
  return new Update(metadata);
};

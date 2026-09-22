import { open } from "@tauri-apps/plugin-dialog";

/** Open a native folder picker; returns the chosen path, or null if cancelled. */
export const pickDirectory = async (): Promise<string | null> => {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
};

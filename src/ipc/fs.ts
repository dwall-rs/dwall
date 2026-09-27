import { invoke } from "@tauri-apps/api/core";

export const moveDirectory = (source: string, destination: string) =>
  invoke<void>("move_directory_cmd", { source, destination });

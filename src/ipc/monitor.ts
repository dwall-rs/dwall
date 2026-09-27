import { invoke } from "@tauri-apps/api/core";

import type { DisplayMonitor } from "./types";

export const getMonitors = async () =>
  invoke<Record<string, DisplayMonitor>>("get_monitors_cmd");

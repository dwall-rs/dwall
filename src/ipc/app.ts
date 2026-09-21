import { invoke } from "@tauri-apps/api/core";

import type { AppInfo } from "./types";

export const getAppInfo = async () => invoke<AppInfo>("get_app_info");

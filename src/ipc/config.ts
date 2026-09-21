import { invoke } from "@tauri-apps/api/core";

import type { Config } from "./types";

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const writeConfigFile = async (config: Config) =>
  invoke<Config>("write_config_file", { config });

export const openConfigDir = async () => invoke<void>("open_config_dir");

import { invoke } from "@tauri-apps/api/core";

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const writeConfigFile = async (config: Config) =>
  invoke<Config>("write_config_file", { config });

export const openConfigDir = async () => invoke<void>("open_config_dir");

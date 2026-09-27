import { invoke } from "@tauri-apps/api/core";

/** Whether the daemon process is currently running */
export const getEngineStatus = async () => invoke<boolean>("get_engine_status");

/** Start the daemon process */
export const startEngine = async () => invoke<void>("start_engine");

/** Stop the daemon process; returns whether a process was terminated */
export const stopEngine = async () => invoke<boolean>("stop_engine");

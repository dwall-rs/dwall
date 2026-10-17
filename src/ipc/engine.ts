import { invoke } from "@tauri-apps/api/core";

/** Whether the daemon process is currently running */
export const getEngineStatus = async () => invoke<boolean>("get_engine_status");

/**
 * The daemon's most recent error line, when it has one.
 *
 * A running daemon is not necessarily a working one: a failed cycle is retried
 * with a backoff instead of ending the process, so the reason only exists in its
 * log.
 */
export const getDaemonLastError = async () =>
  invoke<string | null>("get_daemon_last_error");

/** Start the daemon process */
export const startEngine = async () => invoke<void>("start_engine");

/** Stop the daemon process; returns whether a process was terminated */
export const stopEngine = async () => invoke<boolean>("stop_engine");

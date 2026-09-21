import { invoke } from "@tauri-apps/api/core";

import type { Network } from "./types";

/** Resolve a raw GitHub URL (e.g. a thumbnail) through the configured mirror */
export const mirrorUrl = async (url: string, network?: Network) =>
  invoke<string>("mirror_url", { url, network });

import { invoke } from "@tauri-apps/api/core";

import type { RandomSelection } from "./types";

/** Today's random-mode selection, or `null` when the daemon isn't running. */
export const getRandomSelection = async () =>
  invoke<RandomSelection | null>("get_random_selection");

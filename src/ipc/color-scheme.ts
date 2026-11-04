import { invoke } from "@tauri-apps/api/core";

import type { ColorSchemePause } from "./types";

/** The colour-scheme pause in force, or `null` when switching is not paused. */
export const getColorSchemePause = async () =>
  invoke<ColorSchemePause | null>("get_color_scheme_pause");

/**
 * Pauses automatic colour switching for `minutes`, or resumes it when `minutes` is 0.
 *
 * The pause also starts on its own when the daemon notices a theme change it did not
 * make, so the window polls this rather than assuming it owns the state.
 */
export const setColorSchemePause = async (minutes: number) =>
  invoke<ColorSchemePause | null>("set_color_scheme_pause", { minutes });

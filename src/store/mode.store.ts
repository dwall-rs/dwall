// Responsibility: the wallpaper mode selected in the UI (fixed/random). Seeded from the config on load;
// changing it only affects the UI until the user applies a theme/pool, which persists it to the config.
import type { Mode } from "@/domain/types";
import { createStore } from "solid-js/store";

interface ModeState {
  mode: Mode;
}

const [modeStore, setModeStore] = createStore<ModeState>({ mode: "fixed" });

const setMode = (mode: Mode) => setModeStore("mode", mode);

export { modeStore, setMode };

// Responsibility: view routing state and the window's own cosmetic preferences
// (persisted in the webview). The sidebar owns navigation; this store is the one
// place that knows which surface is showing.
import type { View } from "@/models/types";
import { createStore } from "solid-js/store";

interface UiState {
  view: View;
  /** Whether the content area draws a border under the title bar. */
  titlebarBorder: boolean;
}

/**
 * Where the window's own preferences are remembered.
 *
 * Not `config.toml`: that file is the daemon's too, and a presentation flag would put a
 * field for one line of CSS into the model both binaries share.
 */
const TITLEBAR_BORDER_KEY = "dwall.ui.titlebar-border";

/**
 * The saved value, read synchronously so the first paint already has it — reading it in
 * an effect would show the window without the border for a frame.
 *
 * Anything other than the literals this module writes counts as "off", and a webview
 * where storage is unavailable keeps the window working instead of failing to load.
 */
const readTitlebarBorder = () => {
  try {
    return globalThis.localStorage?.getItem(TITLEBAR_BORDER_KEY) === "true";
  } catch {
    return false;
  }
};

const [uiStore, setUiStore] = createStore<UiState>({
  view: "wallpaper",
  titlebarBorder: readTitlebarBorder(),
});

/**
 * The surface `toggleView` returns to.
 *
 * The settings page can be opened from any of the three working surfaces (and from the
 * update notification), so closing it has to bring the user back where they were rather
 * than dumping them on the wallpaper page.
 */
let lastWorkingView: Exclude<View, "settings"> = "wallpaper";

const setView = (view: View) => {
  if (uiStore.view !== "settings" && uiStore.view !== view) {
    lastWorkingView = uiStore.view;
  }
  setUiStore("view", view);
};

const toggleView = () =>
  setView(uiStore.view === "settings" ? lastWorkingView : "settings");

/**
 * Turns the border on or off and remembers the choice.
 *
 * The store changes first: if the webview refuses the write the toggle still applies for
 * this session, and the next start falls back to the saved value.
 */
const setTitlebarBorder = (titlebarBorder: boolean) => {
  setUiStore("titlebarBorder", titlebarBorder);

  try {
    globalThis.localStorage?.setItem(
      TITLEBAR_BORDER_KEY,
      String(titlebarBorder),
    );
  } catch {
    // See above: remembering it is a convenience, not part of the toggle working.
  }
};

export { uiStore, setView, toggleView, setTitlebarBorder };

// Responsibility: view routing state — view / mode / theme-library drawer toggle. Switching to the settings view also collapses the drawer.
import type { Mode, View } from "@/domain/types";
import { createStore } from "solid-js/store";

interface UiState {
  view: View;
  mode: Mode;
  libOpen: boolean; // whether the theme library drawer is expanded at the smallest breakpoint (ignored at xl+)
}

const [uiStore, setUiStore] = createStore<UiState>({
  view: "main",
  mode: "fixed",
  libOpen: false,
});

const setView = (view: View) =>
  setUiStore((prev) => ({ ...prev, view, libOpen: false }));

const toggleView = () =>
  setUiStore((prev) => ({
    ...prev,
    view: prev.view === "main" ? "settings" : "main",
    libOpen: false,
  }));

const setMode = (mode: Mode) => setUiStore("mode", mode);

const setLibOpen = (libOpen: boolean) => setUiStore("libOpen", libOpen);

const toggleLib = () => setUiStore("libOpen", (prev) => !prev);

export { uiStore, setView, toggleView, setMode, setLibOpen, toggleLib };

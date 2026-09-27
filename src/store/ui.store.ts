// Responsibility: view routing state — current view + theme-library drawer toggle. Switching to the settings view also collapses the drawer.
import type { View } from "@/models/types";
import { createStore } from "solid-js/store";

interface UiState {
  view: View;
  libOpen: boolean; // whether the theme library drawer is expanded at the smallest breakpoint (ignored at xl+)
}

const [uiStore, setUiStore] = createStore<UiState>({
  view: "main",
  libOpen: false,
});

const toggleView = () =>
  setUiStore((prev) => ({
    ...prev,
    view: prev.view === "main" ? "settings" : "main",
    libOpen: false,
  }));

const setLibOpen = (libOpen: boolean) => setUiStore("libOpen", libOpen);

const toggleLib = () => setUiStore("libOpen", (prev) => !prev);

export { uiStore, toggleView, setLibOpen, toggleLib };

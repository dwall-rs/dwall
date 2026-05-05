/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";

import { AppProvider } from "./contexts";
import App from "./App";

if (import.meta.env.MODE === "production") {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

render(
  () => (
    <AppProvider>
      <App />
    </AppProvider>
  ),
  document.getElementById("root") as HTMLElement,
);

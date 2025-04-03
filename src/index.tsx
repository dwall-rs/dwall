/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import { AppProvider } from "~/contexts";
import "fluent-solid/lib/index.css";
import "./index.scss";

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

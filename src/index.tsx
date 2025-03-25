/* @refresh reload */
import { render } from "solid-js/web";
import "fluent-solid/lib/index.css";
import "./index.scss";
import App from "./App";
import { TranslationsProvider } from "./components/TranslationsContext";

if (import.meta.env.MODE === "production") {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

render(
  () => (
    <TranslationsProvider>
      <App />
    </TranslationsProvider>
  ),
  document.getElementById("root") as HTMLElement,
);

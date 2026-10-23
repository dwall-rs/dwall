/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";

import App from "./App";

if (import.meta.env.MODE === "production") {
  // The window has no browser chrome, so the page menu has nothing to offer it - except
  // over a field or over text the app shows for copying, where paste and copy are the
  // only way to move a value in or out without retyping it.
  document.addEventListener("contextmenu", (event) => {
    const target = event.target;
    if (
      target instanceof Element &&
      target.closest("input, textarea, [data-selectable]")
    ) {
      return;
    }
    event.preventDefault();
  });
}

render(() => <App />, document.getElementById("root") as HTMLElement);

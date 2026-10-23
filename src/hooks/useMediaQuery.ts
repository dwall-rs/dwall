// Responsibility: subscribe to a media query and return a boolean; components use it for decisions that pure CSS cannot express (e.g. auto-closing a drawer).

import { type Accessor, createEffect, createSignal, onCleanup } from "solid-js";

/**
 * Subscribe to a media query.
 *
 * The query is an accessor so a caller whose breakpoint can change re-subscribes;
 * the listener is released with onCleanup, because createEffect ignores a returned
 * cleanup and the old listener would otherwise outlive the query it watched.
 */
export function useMediaQuery(query: Accessor<string>): Accessor<boolean> {
  const [matches, setMatches] = createSignal(false);
  createEffect(() => {
    const mql = window.matchMedia(query());
    const onChange = () => setMatches(mql.matches);
    onChange();
    mql.addEventListener("change", onChange);
    onCleanup(() => mql.removeEventListener("change", onChange));
  });

  return matches;
}

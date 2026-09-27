/* ===== src/hooks/useMediaQuery.ts ===== */
// Responsibility: subscribe to a media query and return a boolean; components use it for decisions that pure CSS cannot express (e.g. auto-closing a drawer).

import { type Accessor, createEffect, createSignal } from "solid-js";

export function useMediaQuery(query: string): Accessor<boolean> {
  const [matches, setMatches] = createSignal(
    typeof window !== "undefined" && window.matchMedia(query).matches,
  );
  createEffect(() => {
    const mql = window.matchMedia(query);
    const onChange = () => setMatches(mql.matches);
    onChange();
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, [query]);

  return matches;
}

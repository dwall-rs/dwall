export const detectColorMode = (): ColorMode =>
  window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "DARK"
    : "LIGHT";

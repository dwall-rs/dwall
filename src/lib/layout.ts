// Responsibility: single source of truth for the layout system — minimum
// window size + media query strings, so magic numbers don't scatter.
export const MIN_WINDOW = { width: 1024, height: 640 } as const;
/** Start of the standard tier: theme library goes from drawer back to an inline third column. */
export const MQ_XL = "(min-width: 1280px)";
/** The theme-library drawer toggle. Closing the drawer puts focus back on it, so the view looks it up by id rather than threading a ref through the tooltip. */
export const LIBRARY_TOGGLE_ID = "library-toggle";
/** Minimum window width query (prompted below this). */
export const MQ_MIN = `(min-width: ${MIN_WINDOW.width}px)`;
/** Low-height window: hide the solar arc, giving the height back to the preview. */
export const MQ_SHORT = "(max-height: 720px)";

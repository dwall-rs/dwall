// Responsibility: single source of truth for the layout system — the media query
// strings, so magic numbers don't scatter.
/** Start of the standard tier: theme library goes from drawer back to an inline third column. */
export const MQ_XL = "(min-width: 1280px)";
/** The theme-library drawer toggle. Closing the drawer puts focus back on it, so the view looks it up by id rather than threading a ref through the tooltip. */
export const LIBRARY_TOGGLE_ID = "library-toggle";

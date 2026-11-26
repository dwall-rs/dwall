// Responsibility: when the preview plate has to animate between two frames.
//
// The plate keeps the outgoing frame on screen until the incoming one has decoded, so
// the comparison is about identity, not just the image index: two themes can both have
// a frame 5, and swapping them must still be a transition (and the outgoing layer must
// keep the old theme, not resolve frame 5 against the new one).
import type { Wallpaper } from "@/models/types";

/** A frame together with the theme it belongs to. */
export interface DisplayedFrame {
  themeId: string;
  wallpaper: Wallpaper;
}

/** Whether `next` is already what the plate shows for `themeId`. */
export const isSameDisplayedFrame = (
  current: DisplayedFrame | null,
  themeId: string,
  next: Wallpaper | null,
): boolean =>
  current !== null &&
  next !== null &&
  current.themeId === themeId &&
  current.wallpaper.index === next.index;

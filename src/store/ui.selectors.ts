// Responsibility: derived answers about the window chrome, read from the settings and ui stores.
import { settingsStore } from "./settings.store";
import { uiStore } from "./ui.store";

/**
 * Whether the title bar is painted with Dwall's own background colour.
 *
 * That is what the "title bar follows the Windows theme" switch being off means, and it is
 * the only case where the bar has no colour of its own to set it apart from the content.
 *
 * Explicitly `false`, not "falsy": the window renders before the configuration has been
 * read, and `!config?.field` would claim the title bar matches the content during that
 * moment — briefly offering a switch whose setting is not known yet.
 */
const titleBarMatchesContent = () =>
  settingsStore.config?.title_bar_color_follows_windows_theme === false;

/**
 * Whether the separator is actually drawn.
 *
 * The saved preference alone is not enough: once the title bar follows the Windows theme
 * it has a colour of its own, so the border would be decoration. Keeping the condition here
 * means a preference saved while the border was meaningful cannot stay active, invisibly,
 * after the setting that made it meaningful changed.
 */
const showsTitlebarBorder = () =>
  titleBarMatchesContent() && uiStore.titlebarBorder;

export { titleBarMatchesContent, showsTitlebarBorder };

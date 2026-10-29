// Responsibility: let a store that cannot import settings.store directly ask it to reload.

/**
 * A stop clears the theme in the config file. The open settings form still holds
 * the old draft, so it has to reload before the user sees a theme that the daemon
 * is no longer applying.
 */
type Reload = () => Promise<void>;

let reload: Reload | undefined;

export const registerSettingsReload = (fn: Reload) => {
  reload = fn;
};

export const reloadSettings = async () => {
  await reload?.();
};

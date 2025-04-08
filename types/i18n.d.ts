type TranslationKey =
  | "button-apply"
  | "button-cancel"
  | "button-download"
  | "button-open-log-directory"
  | "button-select-folder"
  | "button-stop"
  | "help-manually-set-coordinates"
  | "label-automatically-retrieve-coordinates"
  | "label-automatically-switch-to-dark-mode"
  | "label-check-interval"
  | "label-github-mirror-template"
  | "label-launch-at-startup"
  | "label-select-monitor"
  | "label-set-lock-screen-wallpaper-simultaneously"
  | "label-source-code"
  | "label-themes-directory"
  | "label-version"
  | "tooltip-check-new-version"
  | "tooltip-new-version-available"
  | "tooltip-open-themes-directory"
  | "tooltip-settings"
  | "message-change-themes-directory"
  | "message-disable-startup-failed"
  | "message-download-cancelled"
  | "message-download-faild"
  | "message-github-star"
  | "message-invalid-number-input"
  | "message-location-permission"
  | "message-coordinates-saved"
  | "message-number-too-small"
  | "message-number-too-large"
  | "message-saving-manual-coordinates"
  | "message-startup-failed"
  | "message-switch-auto-light-dark-mode-failed"
  | "message-switching-to-manual-coordinate-config"
  | "message-themes-directory-moved"
  | "message-update-failed"
  | "message-version-is-the-latest"
  | "placeholder-latitude"
  | "placeholder-longitude"
  | "unit-hour"
  | "unit-second"
  | "title-download-faild"
  | "title-downloading-new-version";

type Translations = Record<
  TranslationKey,
  string | { template: string; params: string[] }
>;

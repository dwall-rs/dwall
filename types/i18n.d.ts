type TranslationKey =
  | "button-apply"
  | "button-stop"
  | "button-download"
  | "button-open-log-directory"
  | "button-select-folder"
  | "label-automatically-retrieve-coordinates"
  | "label-automatically-switch-to-dark-mode"
  | "label-check-interval"
  | "label-github-mirror-template"
  | "label-launch-at-startup"
  | "label-set-lock-screen-wallpaper-simultaneously"
  | "label-themes-directory"
  | "label-version"
  | "tooltip-check-new-version"
  | "tooltip-new-version-available"
  | "tooltip-open-themes-directory"
  | "tooltip-settings"
  | "message-change-themes-directory"
  | "message-startup-failed"
  | "message-disable-startup-failed"
  | "message-download-faild"
  | "message-themes-directory-moved"
  | "message-version-is-the-latest"
  | "unit-second"
  | "title-download-faild"
  | "title-downloading-new-version";

type Translations = Record<
  TranslationKey,
  string | { template: string; params: string[] }
>;

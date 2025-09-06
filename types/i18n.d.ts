interface TranslationTemplate<T extends string> {
  template: string;
  params: T[];
}

interface Translations {
  "button-apply": string;
  "button-cancel": string;
  "button-download": string;
  "button-open-log-directory": string;
  "button-select-folder": string;
  "button-stop": string;

  "help-automatically-switch-to-dark-mode": string;
  "help-github-mirror-template": string;
  "help-launch-at-startup": string;
  "help-manually-set-coordinates": string;
  "help-set-lock-screen-wallpaper-simultaneously": string;
  "help-update-failed": string;

  "label-automatically-retrieve-coordinates": string;
  "label-automatically-switch-to-dark-mode": string;
  "label-check-interval": string;
  "label-github-mirror-template": string;
  "label-launch-at-startup": string;
  "label-select-monitor": string;
  "label-set-lock-screen-wallpaper-simultaneously": string;
  "label-source-code": string;
  "label-themes-directory": string;
  "label-version": string;

  "tooltip-check-new-version": string;
  "tooltip-new-version-available": string;
  "tooltip-open-themes-directory": string;
  "tooltip-settings": string;

  "message-apply-theme-failed": TranslationTemplate<"error">;
  "message-change-themes-directory": TranslationTemplate<"newThemesDirectory">;
  "message-check-interval-updated": TranslationTemplate<"newInterval">;
  "message-disable-startup-failed": TranslationTemplate<"error">;
  "message-download-cancelled": string;
  "message-download-faild": TranslationTemplate<"error">;
  "message-file-size-warning": string;
  "message-github-mirror-template-updated": TranslationTemplate<"newTemplate">;
  "message-github-star": string;
  "message-invalid-number-input": string;
  "message-location-permission": string;
  "message-coordinates-saved": string;
  "message-number-too-large": TranslationTemplate<"max">;
  "message-number-too-small": TranslationTemplate<"min">;
  "message-saving-manual-coordinates": TranslationTemplate<"error">;
  "message-startup-failed": TranslationTemplate<"error">;
  "message-switch-auto-light-dark-mode-failed": TranslationTemplate<"error">;
  "message-switching-to-manual-coordinate-config": TranslationTemplate<"error">;
  "message-themes-directory-moved": TranslationTemplate<"newThemesDirectory">;
  "message-update-available": TranslationTemplate<"version" | "currentVersion">;
  "message-update-failed": TranslationTemplate<"error">;
  "message-version-is-the-latest": string;

  "placeholder-latitude": string;
  "placeholder-longitude": string;

  "unit-hour": string;
  "unit-second": string;

  "title-download-faild": string;
  "title-downloading-new-version": string;
}

type TranslationKey = keyof Translations;

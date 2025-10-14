import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";
import { button } from "fluent-solid/lib/components/button/Button.css";

export const dangerButtonStyles = style({
  selectors: {
    // Use higher specificity selector to override Button base styles
    [`&.${button.base}`]: {
      backgroundColor: themeContract.colorStatusDangerBackground3,
      color: themeContract.colorNeutralForegroundOnBrand,
      borderColor: themeContract.colorTransparentStroke,
    },

    [`&.${button.base}:not(:disabled):hover`]: {
      backgroundColor: themeContract.colorStatusDangerBackground3Hover,
      color: themeContract.colorNeutralForegroundOnBrand,
      borderColor: themeContract.colorTransparentStroke,
    },

    [`&.${button.base}:not(:disabled):hover:active`]: {
      backgroundColor: themeContract.colorStatusDangerBackground3Pressed,
      color: themeContract.colorNeutralForegroundOnBrand,
      borderColor: themeContract.colorTransparentStroke,
    },
  },
});

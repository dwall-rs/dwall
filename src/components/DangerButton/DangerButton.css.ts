import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";

export const dangerButtonStyles = style({
  backgroundColor: themeContract.colorStatusDangerBackground3,
  color: themeContract.colorNeutralForegroundOnBrand,
  borderColor: themeContract.colorTransparentStroke,

  selectors: {
    "&:not(:disabled):hover": {
      backgroundColor: themeContract.colorStatusDangerBackground3Hover,
      color: themeContract.colorNeutralForegroundOnBrand,
      borderColor: themeContract.colorTransparentStroke,
    },

    "&:not(:disabled):hover:active": {
      background: themeContract.colorStatusDangerBackground3Pressed,
      color: themeContract.colorNeutralForegroundOnBrand,
      borderColor: themeContract.colorTransparentStroke,
    },
  },
});

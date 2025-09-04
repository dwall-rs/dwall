import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";

export const sidebarButtons = style({
  flex: 1,
});

export const upgradeButton = style({
  borderColor: themeContract.colorStatusSuccessForeground3,
  color: themeContract.colorStatusSuccessForeground3,

  selectors: {
    "&:not(:disabled):hover": {
      borderColor: themeContract.colorStatusSuccessForeground1,
      color: themeContract.colorStatusSuccessForeground1,
    },
    "&:not(:disabled):hover:active": {
      borderColor: themeContract.colorStatusSuccessForeground2,
      color: themeContract.colorStatusSuccessForeground2,
    },
  },
});

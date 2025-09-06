import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";

export const sidebarButtons = style({
  flex: 1,
});

export const upgradeButton = style({
  color: themeContract.colorStatusSuccessForeground3,

  selectors: {
    "&:not(:disabled):hover": {
      color: themeContract.colorStatusSuccessForeground1,
    },
    "&:not(:disabled):hover:active": {
      color: themeContract.colorStatusSuccessForeground2,
    },
  },
});

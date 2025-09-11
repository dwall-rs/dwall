import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";
import { button } from "fluent-solid/lib/components/button/Button.css";

export const sidebarButtons = style({
  flex: 1,
});

export const upgradeButton = style({
  color: themeContract.colorStatusSuccessForeground3,

  selectors: {
    [`.${button.appearance.transparent}&`]: {
      color: themeContract.colorStatusSuccessForeground3,
    },

    [`.${button.appearance.transparent}&:not(:disabled):hover`]: {
      color: themeContract.colorStatusSuccessForeground1,
    },
    [`.${button.appearance.transparent}&:not(:disabled):hover:active`]: {
      color: themeContract.colorStatusSuccessForeground2,
    },
  },
});

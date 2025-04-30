import { globalStyle, style } from "@vanilla-extract/css";

export const settingsItemContentWrapper = style({});

export const settingsItemContentHelpButton = style({});

globalStyle(`${settingsItemContentHelpButton} span`, {
  fontSize: "0.8rem",
});

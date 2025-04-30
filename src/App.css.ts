import { style } from "@vanilla-extract/css";
import { themeContract } from "fluent-solid/lib/themes";

export const app = style({
  height: "100vh",
});

export const toastMessageLinkLikeButton = style({
  backgroundColor: themeContract.colorTransparentBackground,
  border: "none",
  textDecoration: "underline",
  color: themeContract.colorBrandForegroundLink,
  cursor: "pointer",

  ":hover": {
    color: themeContract.colorBrandForegroundLinkHover,
  },

  ":active": {
    color: themeContract.colorBrandForegroundLinkPressed,
  },
});

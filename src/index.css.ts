import { globalStyle } from "@vanilla-extract/css";
import {
  themeContract,
  vars,
  darkTheme,
  lightTheme,
} from "fluent-solid/lib/themes";

globalStyle(":root", {
  fontFamily: vars.fontFamilyBase,
  color: themeContract.colorNeutralForeground1,
  backgroundColor: themeContract.colorNeutralBackground2,

  "@media": {
    "(prefers-color-scheme: dark)": {
      vars: darkTheme,
    },
    "(prefers-color-scheme: light)": {
      vars: lightTheme,
    },
  },
});

globalStyle("*", {
  userSelect: "none",
  // @ts-expect-error
  WebkitUserDrag: "none",
  MozUserSelect: "none",
  WebkitUserSelect: "none",
  msUserSelect: "none",

  margin: 0,
  padding: 0,
  boxSizing: "border-box",
});

globalStyle("::-webkit-scrollbar", {
  width: "8px",
  height: "8px",
});

globalStyle("::-webkit-scrollbar-track", {
  backgroundColor: themeContract.colorTransparentBackground,
});

globalStyle("::-webkit-scrollbar-thumb", {
  borderRadius: "4px",
  backgroundColor: themeContract.colorNeutralStencil2Alpha,
});

globalStyle("::-webkit-scrollbar-thumb:hover", {
  backgroundColor: themeContract.colorNeutralStencil1Alpha,
});

globalStyle("body", {
  overflow: "hidden",
  margin: 0,
});

globalStyle("#root", {
  display: "flex",
  padding: `${vars.spacingHorizontalXS} ${vars.spacingVerticalL} ${vars.spacingHorizontalL} ${vars.spacingHorizontalL}`,
  height: "100vh",
  width: "100vw",
});

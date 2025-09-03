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
  backgroundColor: themeContract.colorNeutralBackground1,

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
  // @ts-ignore
  WebkitUserDrag: "none",
  MozUserSelect: "none",
  WebkitUserSelect: "none",
  msUserSelect: "none",
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
  height: "100vh",
  width: "100vw",
  display: "flex",
  flexDirection: "column",
  overflow: "hidden",
});

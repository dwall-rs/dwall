import { style, styleVariants } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

export const markdown = style({
  lineHeight: 1.6,
  marginBottom: vars.spacingVerticalL,
  maxHeight: "60vh",
  overflowY: "auto",
});

export const h2 = style({
  margin: 0,
  paddingBottom: vars.spacingVerticalSNudge,
});

export const ul = style({
  margin: `${vars.spacingVerticalS} 0`,
  paddingLeft: vars.spacingHorizontalXXL,
});

export const li = style({
  margin: `${vars.spacingVerticalXS} 0`,
  lineHeight: 1.5,
});

export const blockCode = style({
  backgroundColor: themeContract.colorNeutralBackground2,
  borderRadius: "6px",
  padding: `${vars.spacingVerticalL} ${vars.spacingHorizontalL}`,
  margin: `${vars.spacingVerticalS} 0`,
  overflow: "auto",
  fontFamily: vars.fontFamilyMonospace,
});

export const languageBase = style({
  fontSize: vars.fontSizeBase300,
  fontFamily: vars.fontFamilyMonospace,
});

export const languages = styleVariants({
  typescript: {},
  javascript: {},
  css: {},
  rust: {},
  text: {},
});

export const inlineCode = style({
  backgroundColor: themeContract.colorNeutralBackground3,
  borderRadius: "6px",
  padding: "0.2em 0.4em",
  fontFamily: vars.fontFamilyMonospace,
  fontSize: vars.fontSizeBase300,
});

export const textBlock = style({ margin: `${vars.spacingVerticalS} 0` });

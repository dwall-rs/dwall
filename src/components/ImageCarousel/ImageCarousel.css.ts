import { globalStyle, style } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";
import { appVars } from "~/themes/vars.css";

export const carousel = style({
  width: appVars.contentWidth,
  height: "480px",
  borderRadius: vars.borderRadiusMedium,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

export const wrapper = style({
  width: "480px",
  height: "auto",
  minHeight: "100px",
  maxHeight: "480px",
  position: "relative",
  borderRadius: vars.borderRadiusMedium,
  background: themeContract.colorNeutralBackground1,
  boxShadow: themeContract.shadow4Brand,
  overflow: "hidden",
  transition: `box-shadow ${vars.durationNormal} ease-in-out`,

  selectors: {
    "&:hover": {
      boxShadow: themeContract.shadow8,
    },
  },
});

export const track = style({
  position: "relative",
  width: "100%",
  height: "100%",
});

export const slide = style({
  position: "absolute",
  opacity: 0,
  width: "100%",
  top: "50%",
  transform: "translateY(-50%) scale(1.05)",
  transition: `all ${vars.durationSlow} ease-out`,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

export const activeSlide = style({
  opacity: 1,
  transform: "translateY(-50%) scale(1)",
});

export const image = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

globalStyle(`${image} img`, {
  maxWidth: "100%",
  maxHeight: "100%",
  width: "auto",
  height: "auto",
  objectFit: "contain",
});

export const controls = style({
  opacity: 0,
  transition: `opacity ${vars.durationNormal} ease-in-out`,
});

export const visibleControls = style({
  opacity: 1,
});

export const button = style({
  position: "absolute",
  top: "50%",
  transform: "translateY(-50%)",
  zIndex: 2,
  backgroundColor: themeContract.colorNeutralBackgroundAlpha,
  color: themeContract.colorNeutralForeground1,
  border: "none",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  cursor: "pointer",
  backdropFilter: "blur(4px)",
  transition: `all ${vars.durationNormal} ease`,

  selectors: {
    "&:hover": {
      background: themeContract.colorNeutralBackgroundAlpha2,
      transform: "translateY(-50%) scale(1.05)",
    },
  },
});

export const prevButton = style({
  left: "16px",
});

export const nextButton = style({
  right: "16px",
});

export const indicators = style({
  position: "absolute",
  left: "50%",
  transform: "translateX(-50%)",
  display: "flex",
  gap: vars.spacingHorizontalS,
  zIndex: 2,
  padding: vars.spacingHorizontalS,
  backgroundColor: "rgba(255, 255, 255, 0.2)",
  backdropFilter: "blur(4px)",
  borderRadius: "16px",

  "@media": {
    "(prefers-color-scheme: dark)": {
      backgroundColor: "rgba(0, 0, 0, 0.3)",
    },
  },
});

export const activeIndicator = style({});

export const indicator = style({
  width: "8px",
  height: "8px",
  borderRadius: "50%",
  border: "none",
  backgroundColor: "rgba(255, 255, 255, 0.5)",
  cursor: "pointer",
  padding: 0,
  transition: `all ${vars.durationNormal} ease`,

  "@media": {
    "(prefers-color-scheme: dark)": {
      backgroundColor: "rgba(255, 255, 255, 0.3)",
    },
  },

  selectors: {
    "&:hover": {
      backgroundColor: "rgba(255, 255, 255, 0.8)",

      "@media": {
        "(prefers-color-scheme: dark)": {
          backgroundColor: "rgba(255, 255, 255, 0.5)",
        },
      },
    },

    [`&${activeIndicator}`]: {
      backgroundColor: "#fff",
      transform: "scale(1.2)",

      "@media": {
        "(prefers-color-scheme: dark)": {
          backgroundColor: "rgba(255, 255, 255, 0.7)",
        },
      },
    },
  },
});

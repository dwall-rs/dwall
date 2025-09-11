import { keyframes, style } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

const dialogInAnimation = keyframes({
  from: {
    opacity: 0,
    transform: "scale(0.95)",
  },
  to: {
    opacity: 1,
    transform: "scale(1)",
  },
});

const base = style({
  color: themeContract.colorNeutralForeground1,
  backgroundColor: themeContract.colorNeutralBackground1,
  position: "relative",
  borderRadius: vars.borderRadiusMedium,
  boxShadow: themeContract.shadow16,
  minWidth: "320px",
  maxWidth: "520px",
  padding: 0,
  zIndex: vars.zIndexModal,

  animationName: dialogInAnimation,
  animationDuration: vars.durationNormal,
  animationTimingFunction: "ease-out",
});

const container = style({
  position: "fixed",
  top: 0,
  left: 0,
  width: "100%",
  height: "100%",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: `calc(${vars.zIndexModal} - 1)`,
});

const mask = style({
  position: "fixed",
  top: 0,
  left: 0,
  width: "100%",
  height: "100%",
  backgroundColor: themeContract.colorBackgroundOverlay,
  zIndex: `calc(${vars.zIndexModal} - 1)`,
});

const header = style({
  padding: "16px 24px",
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  userSelect: "none",
});

const title = style({
  margin: 0,
  fontSize: vars.fontSizeBase400,
  lineHeight: vars.lineHeightBase400,
  fontWeight: vars.fontWeightSemibold,
});

const content = style({
  padding: "24px",
  fontSize: vars.fontSizeBase300,
  lineHeight: 1.5,
});

const footer = style({
  padding: "16px 24px",
  borderTop: `1px solid ${themeContract.colorNeutralStroke3}`,
  textAlign: "right",
});

export default {
  base,
  container,
  mask,
  header,
  title,
  content,
  footer,
};

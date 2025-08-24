import { style, styleVariants } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

export const imageContainer = style({
  position: "relative",
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
});

export const spinnerContainer = style({
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
});

export const imageStyle = styleVariants({
  visible: { visibility: "visible" },
  hidden: { visibility: "hidden" },
});

export const errorMessage = style({
  color: themeContract.colorStatusDangerForeground1,
  fontSize: vars.fontSizeBase300,
  textAlign: "center",
});

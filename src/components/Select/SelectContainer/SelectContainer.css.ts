import { style } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

export const container = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: vars.spacingHorizontalXS,
  position: "relative",
});

export const warningMessage = style({
  position: "absolute",
  left: 0,
  top: "100%",
  color: themeContract.colorStatusDangerForeground1,
  marginTop: "4px",
  lineHeight: 1.2,
  whiteSpace: "nowrap",
  fontSize: "0.6rem",
});

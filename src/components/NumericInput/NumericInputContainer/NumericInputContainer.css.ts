import { style } from "@vanilla-extract/css";
import { vars } from "fluent-solid/lib/themes";

export const container = style({
  display: "flex",
  flexDirection: "column",
  gap: vars.spacingVerticalXS,
  position: "relative",
});

export const warningMessage = style({
  position: "absolute",
  left: 0,
  top: "100%",
  color: "#d83b01",
  marginTop: "4px",
  lineHeight: 1.2,
  whiteSpace: "nowrap",
  fontSize: "0.6rem",
});

import { style } from "@vanilla-extract/css";
import { vars } from "fluent-solid/lib/themes";

const downloadContainer = style({
  width: "100%",
  height: "100%",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  gap: vars.spacingVerticalS,
});

const downloadProgress = style({
  position: "absolute",
  bottom: "36px",
});

export default { downloadContainer, downloadProgress };

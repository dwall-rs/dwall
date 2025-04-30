import { style } from "@vanilla-extract/css";
import { vars } from "fluent-solid/lib/themes";

export const inputWrapper = style({
  display: "flex",
  alignItems: "center",
  border: "1px solid #8a8886",
  borderRadius: vars.borderRadiusMedium,
  padding: "0 8px",
  height: "32px",
  background: "#ffffff",
  transition: "all 0.1s ease",

  selectors: {
    "&:hover": {
      borderColor: "#323130",
    },
  },
});

export const focused = style({
  borderColor: "#0078d4",
  boxShadow: "0 0 0 1px #0078d4",
});

export const disabled = style({
  background: "#f3f2f1",
  borderColor: "#c8c6c4",
  cursor: "not-allowed",
});

export const input = style({
  flex: 1,
  outline: "none",
  background: "transparent",
  fontSize: "14px",
  color: "#323130",
  padding: 0,

  selectors: {
    "&::placeholder": {
      color: "#a19f9d",
    },
    "&:disabled": {
      cursor: "not-allowed",
      color: "#a19f9d",
    },
  },
});

export const suffix = style({
  fontSize: "14px",
  color: "#605e5c",
  marginLeft: "4px",
  selectors: {
    [`${disabled} &`]: {
      color: "#a19f9d",
    },
  },
});

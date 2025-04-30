import { style } from "@vanilla-extract/css";

// Base styles
export const flex = style({
  display: "flex",
  boxSizing: "border-box",
  vars: {
    "--flex-grow": "0",
    "--flex-shrink": "0",
  },
});

export const inline = style({
  display: "inline-flex",
});

// Direction variants
export const row = style({
  flexDirection: "row",
});

export const rowReverse = style({
  flexDirection: "row-reverse",
});

export const column = style({
  flexDirection: "column",
});

export const columnReverse = style({
  flexDirection: "column-reverse",
});

// Justify content variants
export const justifyStart = style({
  justifyContent: "flex-start",
});

export const justifyEnd = style({
  justifyContent: "flex-end",
});

export const justifyCenter = style({
  justifyContent: "center",
});

export const justifyBetween = style({
  justifyContent: "space-between",
});

export const justifyAround = style({
  justifyContent: "space-around",
});

export const justifyEvenly = style({
  justifyContent: "space-evenly",
});

export const justifyStretch = style({
  justifyContent: "stretch",
});

// Align items variants
export const alignStart = style({
  alignItems: "flex-start",
});

export const alignEnd = style({
  alignItems: "flex-end",
});

export const alignCenter = style({
  alignItems: "center",
});

export const alignStretch = style({
  alignItems: "stretch",
});

export const alignBaseline = style({
  alignItems: "baseline",
});

// Wrap variants
export const wrap = style({
  flexWrap: "wrap",
});

export const nowrap = style({
  flexWrap: "nowrap",
});

export const wrapReverse = style({
  flexWrap: "wrap-reverse",
});

// Gap variants
export const noGap = style({
  gap: 0,
});

export const gapXS = style({
  gap: "4px",
});

export const gapS = style({
  gap: "8px",
});

export const gapM = style({
  gap: "12px",
});

export const gapL = style({
  gap: "16px",
});

export const gapXL = style({
  gap: "24px",
});

export const gapXXL = style({
  gap: "32px",
});

// Grow and shrink
export const grow = style({
  flexGrow: "var(--flex-grow, 1)",
});

export const noGrow = style({
  flexGrow: 0,
});

export const shrink = style({
  flexShrink: "var(--flex-shrink, 1)",
});

export const noShrink = style({
  flexShrink: 0,
});

export const flexValue = style({
  flexGrow: "var(--flex-grow)",
  flexShrink: "var(--flex-shrink)",
});

// Padding variants
export const p0 = style({
  padding: 0,
});

export const pXS = style({
  padding: "4px",
});

export const pS = style({
  padding: "8px",
});

export const pM = style({
  padding: "12px",
});

export const pL = style({
  padding: "16px",
});

export const pXL = style({
  padding: "24px",
});

export const pXXL = style({
  padding: "32px",
});

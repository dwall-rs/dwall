import { style, globalStyle, createVar } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

export const menuItemColorShadow = createVar();

export const thumbnailsContainer = style({
  flex: "7",
  overflowY: "auto",
  padding: `${vars.spacingVerticalMNudge} ${vars.spacingHorizontalMNudge} ${vars.spacingVerticalMNudge} ${vars.spacingHorizontalXL}`,
});

export const menuItemDisabled = style({});

export const menuItem = style({
  vars: {
    [menuItemColorShadow]: "rgba(0, 0, 0, 0.3)",
  },
  padding: "4px",
  borderRadius: "5px",
  height: "64px",
  width: "64px",
  display: "flex",
  alignItems: "center",
  position: "relative",
  transition: `all ${vars.durationNormal} ease-in-out`,
  backgroundColor: themeContract.colorNeutralBackground6,

  selectors: {
    "&::after": {
      content: '""',
      position: "absolute",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      opacity: 0,
      borderRadius: "5px",
      background:
        "linear-gradient(45deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05))",
      transition: `opacity ${vars.durationNormal} ease-in-out`,
      pointerEvents: "none",
    },

    [`&:not(${menuItemDisabled}):hover`]: {
      background: themeContract.colorNeutralCardBackgroundHover,
      transform: "translateY(-2px)",
      boxShadow: `0 5px 15px ${menuItemColorShadow}`,
    },

    [`&:not(${menuItemDisabled}):hover::after`]: {
      opacity: 1,
    },

    [`&:not(${menuItemDisabled}):hover:active`]: {
      vars: {
        [menuItemColorShadow]: "rgba(0, 0, 0, 0.5)",
      },
      background: themeContract.colorNeutralCardBackgroundPressed,
      boxShadow: `0 3px 10px ${menuItemColorShadow}`,
      scale: "0.95",
    },
  },
});

export const menuItemActive = style({
  vars: {
    [menuItemColorShadow]: "rgba(0, 0, 0, 0.5)",
  },
  backgroundColor: themeContract.colorNeutralCardBackgroundHover,
  transform: "translateY(-2px)",
  boxShadow: `0 3px 10px ${menuItemColorShadow}`,

  selectors: {
    "&::after": {
      opacity: 1,
    },
  },
});

export const menuItemApplied = style({
  position: "relative",
});

export const menuItemAppliedBadge = style({
  position: "absolute",
  right: "4px",
  bottom: "4px",
  width: "16px",
  minWidth: "16px",
  height: "16px",
});

globalStyle(`${menuItem} img`, {
  borderRadius: vars.borderRadiusMedium,
});

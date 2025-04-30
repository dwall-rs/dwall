import { style, keyframes } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

const selectWrapper = style({
  display: "flex",
  alignItems: "center",
  borderRadius: vars.borderRadiusMedium,
  border: `${vars.strokeWidthThin} solid ${themeContract.colorNeutralStroke1}`,
  borderBottomColor: themeContract.colorNeutralStrokeAccessible,
  padding: `${vars.spacingHorizontalNone} ${vars.spacingHorizontalS} ${vars.spacingHorizontalNone} ${vars.spacingHorizontalM}`,
  height: "32px",
  background: themeContract.colorNeutralBackground1,
  transition: `border-color ${vars.durationFast} ease-in-out, background ${vars.durationFast} ease-in-out`,
  cursor: "pointer",
  position: "relative",
  minWidth: "240px",
  fontFamily: vars.fontFamilyBase,
  fontSize: vars.fontSizeBase300,
  lineHeight: vars.lineHeightBase300,

  selectors: {
    "&:hover": {
      backgroundColor: themeContract.colorNeutralBackground1Hover,
    },
    "&:active": {
      backgroundColor: themeContract.colorNeutralBackground1Pressed,
    },
    "&::after": {
      boxSizing: "border-box",
      content: "''",
      position: "absolute",
      left: "-1px",
      bottom: "-1px",
      right: "-1px",
      height: vars.borderRadiusMedium,
      borderBottomLeftRadius: vars.borderRadiusMedium,
      borderBottomRightRadius: vars.borderRadiusMedium,
      borderBottom: `${vars.strokeWidthThick} solid ${themeContract.colorCompoundBrandStroke}`,
      clipPath: "inset(calc(100% - 2px) 0px 0px)",
      transform: "scaleX(0)",
      transitionProperty: "transform",
      transitionDuration: vars.durationUltraFast,
      transitionDelay: vars.curveAccelerateMid,
    },
  },
});

const selectWrapperFocused = style({
  outline: `${themeContract.colorTransparentStroke} solid ${vars.strokeWidthThick}`,

  selectors: {
    "&::after": {
      transform: "scaleX(1)",
      transitionProperty: "transform",
      transitionDuration: vars.durationNormal,
      transitionDelay: vars.curveDecelerateMid,
    },
  },
});

const selectWrapperDisabled = style({
  cursor: "not-allowed",
  backgroundColor: themeContract.colorNeutralBackground3,
  borderColor: themeContract.colorNeutralStrokeDisabled,

  selectors: {
    "&::after": {
      borderBottomColor: themeContract.colorNeutralStrokeDisabled,
    },
  },
});

const value = style({
  flex: 1,
  fontSize: "inherit",
  color: themeContract.colorNeutralForeground1,
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
});

const placeholder = style({
  flex: 1,
  fontSize: "inherit",
  color: themeContract.colorNeutralForeground4,
});

const arrow = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  marginLeft: vars.spacingHorizontalS,
  color: themeContract.colorNeutralForeground3,
  transition: `color ${vars.durationFast} ease-in-out, transform ${vars.durationNormal} ${vars.curveDecelerateMax}`,
});

const arrowOpen = style({
  transform: "rotate(180deg)",
  color: themeContract.colorNeutralForeground2,
});

const dropdown = style({
  position: "absolute",
  top: "calc(100% + 4px)",
  left: 0,
  width: "100%",
  maxHeight: "300px",
  overflowY: "auto",
  backgroundColor: themeContract.colorNeutralBackground1,
  border: `${vars.strokeWidthThin} solid ${themeContract.colorNeutralStroke1}`,
  borderRadius: vars.borderRadiusXLarge,
  zIndex: vars.zIndexDropdown,
  display: "none",
  boxShadow: themeContract.shadow16,
  padding: `${vars.spacingVerticalXS} ${vars.spacingVerticalNone}`,

  selectors: {
    "&::-webkit-scrollbar": {
      width: "6px",
    },
    "&::-webkit-scrollbar-thumb": {
      background: themeContract.colorNeutralForeground3,
      borderRadius: vars.borderRadiusLarge,
    },
    "&::-webkit-scrollbar-track": {
      backgroundColor: themeContract.colorTransparentBackground,
    },
  },
});

const dropdownInAnimation = keyframes({
  from: {
    opacity: 0,
    transform: "translateY(-4px)",
  },
  to: {
    opacity: 1,
    transform: "translateY(0)",
  },
});

const dropdownOpen = style({
  display: "flex",
  flexDirection: "column",
  justifyContent: "center",
  gap: vars.spacingVerticalXS,
  animation: `${dropdownInAnimation} ${vars.durationNormal} ${vars.curveEasyEase}`,
});

const option = style({
  padding: `0 ${vars.spacingHorizontalM}`,
  fontSize: "inherit",
  color: themeContract.colorNeutralForeground1,
  cursor: "pointer",
  transition: `background ${vars.durationFaster} ease-in-out, color ${vars.durationFaster} ease-in-out`,
  height: "32px",
  display: "flex",
  alignItems: "center",
  position: "relative",
  borderRadius: vars.borderRadiusMedium,
  margin: `${vars.spacingVerticalNone} ${vars.spacingHorizontalXS}`,

  selectors: {
    "&:hover": {
      background: themeContract.colorSubtleBackgroundSelected,
      color: themeContract.colorNeutralForeground1Hover,
    },
    "&:active": {
      background: themeContract.colorSubtleBackgroundPressed,
      color: themeContract.colorNeutralForeground1Pressed,
    },
  },
});

const optionSelected = style({
  background: themeContract.colorSubtleBackgroundSelected,
  color: themeContract.colorNeutralForeground1Selected,
  fontWeight: vars.fontWeightSemibold,

  selectors: {
    "&::before": {
      content: "''",
      position: "absolute",
      left: 0,
      top: "7px",
      bottom: "7px",
      width: vars.strokeWidthThicker,
      background: themeContract.colorCompoundBrandStroke,
      borderRadius: vars.borderRadiusSmall,
    },
    "&:hover": {
      background: themeContract.colorSubtleBackgroundHover,
    },
    "&:active": {
      background: themeContract.colorSubtleBackgroundPressed,
    },
  },
});

const optionHighlighted = style({
  background: themeContract.colorSubtleBackgroundHover,
  color: themeContract.colorNeutralForeground1Hover,
});

const optionHighlightedIndicator = style({
  selectors: {
    "&::before": {
      content: "''",
      position: "absolute",
      left: 0,
      top: "4px",
      bottom: "4px",
      width: vars.strokeWidthThick,
      background: themeContract.colorNeutralStroke1,
      borderRadius: `${vars.borderRadiusNone} ${vars.borderRadiusSmall} ${vars.borderRadiusSmall} ${vars.borderRadiusNone}`,
      opacity: 0.6,
    },
  },
});

const optionDisabled = style({
  color: themeContract.colorNeutralForegroundDisabled,
  background: "transparent",
  cursor: "not-allowed",
});

export default {
  selectWrapper,
  selectWrapperFocused,
  selectWrapperDisabled,
  value,
  placeholder,
  arrow,
  arrowOpen,
  dropdown,
  dropdownOpen,
  option,
  optionSelected,
  optionHighlighted,
  optionHighlightedIndicator,
  optionDisabled,
};

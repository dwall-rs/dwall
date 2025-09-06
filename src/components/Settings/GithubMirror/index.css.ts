import { style } from "@vanilla-extract/css";
import { themeContract, vars } from "fluent-solid/lib/themes";

export const code = style({
  fontFamily: vars.fontFamilyMonospace,
  borderColor: themeContract.colorNeutralStroke1,
  borderStyle: "solid",
  borderWidth: vars.strokeWidthThin,
  borderRadius: vars.borderRadiusMedium,
  backgroundColor: themeContract.colorNeutralBackground2,
  padding: `${vars.spacingHorizontalXXS} ${vars.spacingVerticalXS}`,
  display: "inline-block",
});

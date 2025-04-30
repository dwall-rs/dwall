import { mergeProps, splitProps } from "solid-js";
import type { JSX, Component } from "solid-js";
import * as styles from "./Flex.css";

export interface FlexProps {
  /** Whether to display inline */
  inline?: boolean;
  /** Child elements */
  children?: JSX.Element;
  /** Custom class name */
  class?: string;
  /** Custom styles */
  style?: JSX.CSSProperties;
  /** Layout direction */
  direction?: "row" | "column" | "row-reverse" | "column-reverse";
  /** Main axis alignment */
  justify?:
    | "start"
    | "end"
    | "center"
    | "between"
    | "around"
    | "evenly"
    | "stretch";
  /** Cross axis alignment */
  align?: "start" | "end" | "center" | "stretch" | "baseline";
  /** Whether to wrap */
  wrap?: "wrap" | "nowrap" | "wrap-reverse";
  /** Gap between elements */
  gap?: 0 | "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl";
  /** Whether to fill parent container */
  fill?: boolean;
  /** Whether to auto grow */
  grow?: boolean | number;
  /** Whether to auto shrink, can be boolean or number */
  shrink?: boolean | number;
  /** Padding, supports preset sizes */
  padding?: "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl" | string;
  /** Top padding, supports preset sizes or custom values */
  paddingTop?: "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl" | string | number;
  /** Right padding, supports preset sizes or custom values */
  paddingRight?: "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl" | string | number;
  /** Bottom padding, supports preset sizes or custom values */
  paddingBottom?: "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl" | string | number;
  /** Left padding, supports preset sizes or custom values */
  paddingLeft?: "0" | "xs" | "s" | "m" | "l" | "xl" | "xxl" | string | number;
  /** Click event handler */
  onClick?: JSX.EventHandlerUnion<HTMLDivElement, MouseEvent>;
}

/**
 * Flex Component - FluentUI-style flexible layout container
 *
 * Provides convenient flex layout capabilities, supporting common properties like direction, alignment, and spacing
 */
const Flex: Component<FlexProps> = (props) => {
  const defaultProps: Partial<FlexProps> = {
    direction: "row",
    justify: "start",
    align: "start",
    wrap: "nowrap",
    gap: "m",
    fill: false,
    grow: false,
    shrink: false,
    padding: "0" as FlexProps["padding"],
    paddingTop: undefined,
    paddingRight: undefined,
    paddingBottom: undefined,
    paddingLeft: undefined,
  };

  const merged = mergeProps(defaultProps, props);
  const [local, others] = splitProps(merged, [
    "inline",
    "children",
    "class",
    "style",
    "direction",
    "justify",
    "align",
    "wrap",
    "gap",
    "fill",
    "grow",
    "shrink",
    "padding",
    "paddingTop",
    "paddingRight",
    "paddingBottom",
    "paddingLeft",
  ]);

  // Build class names
  const getDirectionClass = () => {
    switch (local.direction) {
      case "row":
        return styles.row;
      case "column":
        return styles.column;
      case "row-reverse":
        return styles.rowReverse;
      case "column-reverse":
        return styles.columnReverse;
      default:
        return styles.row;
    }
  };

  const getJustifyClass = () => {
    switch (local.justify) {
      case "start":
        return styles.justifyStart;
      case "end":
        return styles.justifyEnd;
      case "center":
        return styles.justifyCenter;
      case "between":
        return styles.justifyBetween;
      case "around":
        return styles.justifyAround;
      case "evenly":
        return styles.justifyEvenly;
      case "stretch":
        return styles.justifyStretch;
      default:
        return styles.justifyStart;
    }
  };

  const getAlignClass = () => {
    switch (local.align) {
      case "start":
        return styles.alignStart;
      case "end":
        return styles.alignEnd;
      case "center":
        return styles.alignCenter;
      case "stretch":
        return styles.alignStretch;
      case "baseline":
        return styles.alignBaseline;
      default:
        return styles.alignStart;
    }
  };

  const getWrapClass = () => {
    switch (local.wrap) {
      case "wrap":
        return styles.wrap;
      case "nowrap":
        return styles.nowrap;
      case "wrap-reverse":
        return styles.wrapReverse;
      default:
        return styles.nowrap;
    }
  };

  const getGapClass = () => {
    switch (local.gap) {
      case 0:
      case "0":
        return styles.noGap;
      case "xs":
        return styles.gapXS;
      case "s":
        return styles.gapS;
      case "m":
        return styles.gapM;
      case "l":
        return styles.gapL;
      case "xl":
        return styles.gapXL;
      case "xxl":
        return styles.gapXXL;
      default:
        return styles.gapM;
    }
  };

  const getPaddingClass = () => {
    switch (local.padding) {
      case "0":
        return styles.p0;
      case "xs":
        return styles.pXS;
      case "s":
        return styles.pS;
      case "m":
        return styles.pM;
      case "l":
        return styles.pL;
      case "xl":
        return styles.pXL;
      case "xxl":
        return styles.pXXL;
      default:
        return styles.p0;
    }
  };

  const getGrowShrinkClass = () => {
    const classes = [];

    // Handle grow property
    if (typeof local.grow === "number") {
      classes.push(styles.flexValue);
    } else if (local.grow) {
      classes.push(styles.grow);
    } else {
      classes.push(styles.noGrow);
    }

    // Handle shrink property
    if (typeof local.shrink === "number") {
      classes.push(styles.flexValue);
    } else if (local.shrink) {
      classes.push(styles.shrink);
    } else {
      classes.push(styles.noShrink);
    }

    return classes.join(" ");
  };

  const getInlineClass = () => {
    return local.inline ? styles.inline : "";
  };

  const getClasses = () => {
    return [
      styles.flex,
      getInlineClass(),
      getDirectionClass(),
      getJustifyClass(),
      getAlignClass(),
      getWrapClass(),
      getGapClass(),
      getPaddingClass(),
      getGrowShrinkClass(),
      local.class,
    ]
      .filter(Boolean)
      .join(" ");
  };

  // Build styles
  const getStyles = () => {
    const customStyles = { ...local.style };

    // Set fill property
    if (local.fill) {
      customStyles.width = "100%";
      customStyles.height = "100%";
    }

    // Set grow CSS variable
    if (typeof local.grow === "number") {
      customStyles["--flex-grow"] = local.grow;
    }

    // Set shrink CSS variable
    if (typeof local.shrink === "number") {
      customStyles["--flex-shrink"] = local.shrink;
    }

    // Size preset mapping table
    const paddingSizes = {
      "0": "0",
      xs: "4px",
      s: "8px",
      m: "12px",
      l: "16px",
      xl: "24px",
      xxl: "32px",
    };

    // Process padding for each direction, higher priority than general padding
    const processPadding = (
      prop: "paddingTop" | "paddingRight" | "paddingBottom" | "paddingLeft",
      cssProp:
        | "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left",
    ) => {
      const value = local[prop] as keyof typeof paddingSizes;
      if (value !== undefined) {
        if (typeof value === "number") {
          customStyles[cssProp] = `${value}px`;
        } else if (paddingSizes[value]) {
          customStyles[cssProp] = paddingSizes[value];
        } else {
          customStyles[cssProp] = value;
        }
        return true; // Mark if this direction's padding is set
      }
      return false;
    };

    // Process padding for each direction
    const hasTopPadding = processPadding("paddingTop", "padding-top");
    const hasRightPadding = processPadding("paddingRight", "padding-right");
    const hasBottomPadding = processPadding("paddingBottom", "padding-bottom");
    const hasLeftPadding = processPadding("paddingLeft", "padding-left");

    // Only apply general padding when no individual direction padding is set
    if (
      !hasTopPadding &&
      !hasRightPadding &&
      !hasBottomPadding &&
      !hasLeftPadding &&
      local.padding !== undefined
    ) {
      const padding = local.padding as keyof typeof paddingSizes;
      if (paddingSizes[padding]) {
        // If using preset size
        customStyles.padding = paddingSizes[padding];
      } else {
        // If using custom value (like "10px 20px")
        customStyles.padding = local.padding;
      }
    }

    return customStyles;
  };

  return (
    <div class={getClasses()} style={getStyles()} {...others}>
      {local.children}
    </div>
  );
};

export default Flex;

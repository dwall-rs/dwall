import type { FlexProps } from "~/components/Flex";
import { lazy } from "solid-js";

export const LazyFlex = lazy(() => import("~/components/Flex"));

export const LazySpace = (props: Omit<FlexProps, "inline" | "align">) => {
  return (
    <LazyFlex inline align="center" {...props}>
      {props.children}
    </LazyFlex>
  );
};

export const LazyButton = lazy(
  () => import("fluent-solid/lib/components/button/Button"),
);

export const LazyProgress = lazy(
  () => import("fluent-solid/lib/components/progress/Progress"),
);

export const LazyLabel = lazy(
  () => import("fluent-solid/lib/components/label/Label"),
);

export const LazySwitch = lazy(
  () => import("fluent-solid/lib/components/switch/Switch"),
);

export const LazyInput = lazy(
  () => import("fluent-solid/lib/components/input/Input"),
);

export const LazySlider = lazy(
  () => import("fluent-solid/lib/components/slider/Slider"),
);

export const LazyBadge = lazy(
  () => import("fluent-solid/lib/components/badge/Badge"),
);

export const LazySpinner = lazy(
  () => import("fluent-solid/lib/components/spinner/Spinner"),
);

export const LazyTooltip = lazy(
  () => import("fluent-solid/lib/components/tooltip/Tooltip"),
);

export const LazyDivider = lazy(
  () => import("fluent-solid/lib/components/divider/Divider"),
);

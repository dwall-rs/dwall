import { lazy } from "solid-js";

export const LazyFlex = lazy(
  () => import("alley-components/lib/components/flex"),
);

export const LazySpace = lazy(
  () => import("alley-components/lib/components/space"),
);

// ------ fluent -----

export const LazyButton = lazy(
  () => import("fluent-solid/lib/components/button"),
);

export const LazyProgress = lazy(
  () => import("fluent-solid/lib/components/progress"),
);

export const LazyLabel = lazy(
  () => import("fluent-solid/lib/components/label"),
);

export const LazySwitch = lazy(
  () => import("fluent-solid/lib/components/switch"),
);

export const LazyInput = lazy(
  () => import("fluent-solid/lib/components/input"),
);

export const LazySlider = lazy(
  () => import("fluent-solid/lib/components/slider"),
);

export const LazyBadge = lazy(
  () => import("fluent-solid/lib/components/badge/badge"),
);

export const LazySpinner = lazy(
  () => import("fluent-solid/lib/components/spinner"),
);

export const LazyTooltip = lazy(
  () => import("fluent-solid/lib/components/tooltip"),
);

export const LazyDivider = lazy(
  () => import("fluent-solid/lib/components/divider"),
);

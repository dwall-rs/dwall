import { lazy } from "solid-js";

export const LazyFlex = lazy(
  () => import("alley-components/lib/components/flex"),
);

export const LazyTextArea = lazy(
  () => import("alley-components/lib/components/input/text-area"),
);

export const LazyCol = lazy(
  () => import("alley-components/lib/components/col"),
);

export const LazyRow = lazy(
  () => import("alley-components/lib/components/row"),
);

export const LazyDivider = lazy(
  () => import("alley-components/lib/components/divider"),
);

export const LazySpace = lazy(
  () => import("alley-components/lib/components/space"),
);
export const LazySpaceCompact = lazy(
  () => import("alley-components/lib/components/space/compact"),
);

export const LazyTooltip = lazy(
  () => import("alley-components/lib/components/tooltip"),
);

export const LazyTypography = lazy(
  () => import("alley-components/lib/components/typography"),
);

export const LazyText = lazy(
  () => import("alley-components/lib/components/typography/text"),
);

export const LazyTag = lazy(
  () => import("alley-components/lib/components/tag"),
);

export const LazyDialog = lazy(
  () => import("alley-components/lib/components/dialog"),
);

export const LazyToast = lazy(
  () => import("alley-components/lib/components/toast"),
);

export const LazyAlert = lazy(
  () => import("alley-components/lib/components/alert"),
);

export const LazyInputNumber = lazy(
  () => import("alley-components/lib/components/input-number"),
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

import type { Accessor, ParentProps } from "solid-js";
import type { Rect } from "~/lib";

export interface TooltipProps extends ParentProps {
  open?: boolean;

  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;

  openDelay?: number;

  closeDelay?: number;

  disabled?: boolean;
}

export interface TooltipContextValue {
  open: Accessor<boolean>;
  disabled: Accessor<boolean>;

  contentId: string;
  reference: Accessor<Element | undefined>;
  setReference: (el: Element) => void;
  floating: Accessor<HTMLElement | undefined>;
  setFloating: (el: HTMLElement | undefined) => void;

  requestOpen: () => void;

  requestClose: () => void;

  openImmediate: () => void;

  closeImmediate: () => void;

  keepOpen: () => void;

  pendingSlideFrom: Accessor<Rect | undefined>;
  clearPendingSlideFrom: () => void;

  suppressExitAnimation: Accessor<boolean>;
  consumeSuppressExitAnimation: () => boolean;
}

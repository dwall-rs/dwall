import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  type Accessor,
} from "solid-js";
import type {
  Middleware,
  MiddlewareData,
  Placement,
  ReferenceElement,
  Strategy,
} from "./types";
import { computePosition } from "./core/compute-position";
import {
  autoUpdate as attachAutoUpdate,
  type AutoUpdateOptions,
} from "./auto-update";

type MaybeAccessor<T> = T | Accessor<T>;

function unwrap<T>(value: MaybeAccessor<T> | undefined, fallback: T): T {
  if (value === undefined) return fallback;
  return typeof value === "function" ? (value as Accessor<T>)() : value;
}

export interface CreatePositionerOptions {
  /** Desired initial/preferred placement, default 'bottom' */
  placement?: MaybeAccessor<Placement>;
  /** Positioning strategy: 'absolute' requires the floating to be under body with no non-static positioned ancestor; use 'fixed' when unsure */
  strategy?: MaybeAccessor<Strategy>;
  /** Middleware array, executed in order, e.g. [offset(8), flip(), shift({padding: 8})] */
  middleware?: MaybeAccessor<Middleware[]>;
  /** Whether to recompute automatically on scroll/size changes, default true; fine-grained options can also be passed */
  autoUpdate?: boolean | AutoUpdateOptions;
}

export interface Positioner {
  /** x of the floating element in its positioning coordinate system (fine-grained signal, updates only when the value changes) */
  x: Accessor<number>;
  /** y of the floating element in its positioning coordinate system (fine-grained signal, updates only when the value changes) */
  y: Accessor<number>;
  /** Final placement after middleware (e.g. flip) adjustments */
  placement: Accessor<Placement>;
  /** The positioning strategy actually in effect */
  strategy: Accessor<Strategy>;
  /** Additional data produced by each middleware, e.g. the arrow's offset */
  middlewareData: Accessor<MiddlewareData>;
  /** Whether at least one positioning computation has completed; useful for hiding the floating element before the first render to avoid flashing */
  isPositioned: Accessor<boolean>;
  /** Manually trigger a recomputation */
  update: () => void;
  /** Convenience style object that can be spread directly onto style */
  floatingStyles: Accessor<{
    position: Strategy;
    top: string;
    left: string;
    transform: string;
  }>;
}

/**
 * Solid's fine-grained floating positioning primitive.
 *
 * Pass in the reference / floating element accessors and get back a set of
 * independent reactive signals (x, y, placement, etc. update separately — a
 * change in one does not re-render the others), so upper-layer components
 * (tooltip / dropdown / popover) only subscribe to the signals they care about.
 *
 * @example
 * ```tsx
 * const [reference, setReference] = createSignal<HTMLElement>();
 * const [floating, setFloating] = createSignal<HTMLElement>();
 * const pos = createPositioner(reference, floating, {
 *   placement: "bottom-start",
 *   middleware: [offset(8), flip(), shift({ padding: 8 })],
 * });
 *
 * <button ref={setReference}>trigger</button>
 * <Show when={open()}>
 *   <Portal>
 *     <div ref={setFloating} style={pos.floatingStyles()}>content</div>
 *   </Portal>
 * </Show>
 * ```
 */
export function createPositioner(
  reference: Accessor<ReferenceElement | null | undefined>,
  floating: Accessor<HTMLElement | null | undefined>,
  options: CreatePositionerOptions = {},
): Positioner {
  const [x, setX] = createSignal(0);
  const [y, setY] = createSignal(0);
  const [placement, setPlacement] = createSignal<Placement>(
    unwrap(options.placement, "bottom"),
  );
  const [strategy, setStrategy] = createSignal<Strategy>(
    unwrap(options.strategy, "absolute"),
  );
  const [middlewareData, setMiddlewareData] = createSignal<MiddlewareData>({});
  const [isPositioned, setIsPositioned] = createSignal(false);

  const update = () => {
    const refEl = reference();
    const floatEl = floating();
    if (!refEl || !floatEl) {
      setIsPositioned(false);
      return;
    }

    const result = computePosition(refEl, floatEl, {
      placement: unwrap(options.placement, "bottom"),
      strategy: unwrap(options.strategy, "absolute"),
      middleware: unwrap(options.middleware, []),
    });

    // Each setter only triggers downstream updates when the value truly
    // changes (Solid's default signal behavior) — the key to "fine-grained":
    // nodes subscribed only to x never re-run because middlewareData changed.
    setX(result.x);
    setY(result.y);
    setPlacement(result.placement);
    setStrategy(result.strategy);
    setMiddlewareData(result.middlewareData);
    setIsPositioned(true);
  };

  createEffect(() => {
    const refEl = reference();
    const floatEl = floating();
    // Read eagerly to establish dependencies: if placement/strategy/middleware
    // are reactive accessors, changes re-enter this effect, recompute, and
    // re-attach autoUpdate.
    unwrap(options.placement, "bottom");
    unwrap(options.strategy, "absolute");
    unwrap(options.middleware, []);

    if (!refEl || !floatEl) {
      setIsPositioned(false);
      return;
    }

    update();

    if (options.autoUpdate !== false) {
      const cleanup = attachAutoUpdate(
        refEl,
        floatEl,
        update,
        typeof options.autoUpdate === "object" ? options.autoUpdate : {},
      );
      onCleanup(cleanup);
    }
  });

  const floatingStyles = createMemo(() => ({
    position: strategy(),
    top: "0px",
    left: "0px",
    // Assign via transform instead of top/left to avoid triggering layout — better performance
    transform: `translate(${Math.round(x())}px, ${Math.round(y())}px)`,
  }));

  return {
    x,
    y,
    placement,
    strategy,
    middlewareData,
    isPositioned,
    update,
    floatingStyles,
  };
}

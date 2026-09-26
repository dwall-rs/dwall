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
  /** 期望的初始/首选摆放位置，默认 'bottom' */
  placement?: MaybeAccessor<Placement>;
  /** 定位方式：'absolute' 需要 floating 挂在 body 下且无非 static 定位祖先；不确定就用 'fixed' */
  strategy?: MaybeAccessor<Strategy>;
  /** middleware 数组，顺序执行，如 [offset(8), flip(), shift({padding: 8})] */
  middleware?: MaybeAccessor<Middleware[]>;
  /** 是否在滚动/尺寸变化时自动重新计算，默认 true；也可传细粒度配置 */
  autoUpdate?: boolean | AutoUpdateOptions;
}

export interface Positioner {
  /** floating 元素相对定位坐标系的 x（细粒度信号，只在数值变化时更新） */
  x: Accessor<number>;
  /** floating 元素相对定位坐标系的 y（细粒度信号，只在数值变化时更新） */
  y: Accessor<number>;
  /** 经过 middleware（如 flip）调整后的最终 placement */
  placement: Accessor<Placement>;
  /** 实际生效的定位策略 */
  strategy: Accessor<Strategy>;
  /** 各 middleware 产出的附加数据，例如 arrow 的偏移量 */
  middlewareData: Accessor<MiddlewareData>;
  /** 是否已完成至少一次定位计算，可用于首次渲染前隐藏 floating 元素避免闪烁 */
  isPositioned: Accessor<boolean>;
  /** 手动触发一次重新计算 */
  update: () => void;
  /** 可直接展开到 style 上的便捷样式对象 */
  floatingStyles: Accessor<{
    position: Strategy;
    top: string;
    left: string;
    transform: string;
  }>;
}

/**
 * Solid 的细粒度浮窗定位 primitive。
 *
 * 传入 reference / floating 两个元素访问器，返回一组独立的响应式信号
 * （x、y、placement 等分别更新，不会因为其中一个变化而让另一个也重新渲染），
 * 上层组件（tooltip / dropdown / popover）只需订阅自己关心的信号。
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
 *     <div ref={setFloating} style={pos.floatingStyles()}>内容</div>
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

    // 每个 setter 只有在值真正变化时才会触发下游更新（Solid 信号默认行为），
    // 这就是“细粒度”的关键：只订阅 x 的节点不会因为 middlewareData 变化而重跑。
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
    // 主动读取，建立依赖：placement/strategy/middleware 若是响应式访问器，
    // 变化时也会重新进入这个 effect 并重新计算 + 重新挂 autoUpdate。
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
    // 用 transform 而非 top/left 赋值，避免触发 layout，性能更好
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

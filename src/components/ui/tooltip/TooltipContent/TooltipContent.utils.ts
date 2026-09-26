import {
  offset,
  flip,
  shift,
  arrow,
  hide,
  containingBlockOffset,
  getSide,
  getAlignment,
  type Middleware,
  type Placement,
  type Side,
  type Alignment,
} from "~/lib";

/**
 * 把 shadcn/Radix 风格的 side + align 拆分参数，转换成核心库统一用的
 * `Placement` 字符串（比如 side='top' + align='start' → 'top-start'）。
 * align 是 'center'（默认对齐、不偏移）时不带后缀，直接是 side 本身。
 */
export function toPhysicalSide(
  side: Side | "inline-start" | "inline-end",
  dir?: string,
): Side {
  if (side === "inline-start") {
    return dir === "rtl" ? "right" : "left";
  }
  if (side === "inline-end") {
    return dir === "rtl" ? "left" : "right";
  }
  return side;
}

export function toPlacement(
  side: Side | "inline-start" | "inline-end",
  align: Alignment | "center",
  dir?: string,
): Placement {
  const physical = toPhysicalSide(side, dir);
  return align === "center" ? physical : (`${physical}-${align}` as Placement);
}

/**
 * 根据最终生效的 placement，算出缩放动画应该以哪个点为锚点（CSS transform-origin）。
 * 锚点应该落在"贴近 trigger 的那条边"上——比如 placement 是 'top'（content 在
 * trigger 上方），锚点就该是 content 自己的底边，缩放动画看起来才会像是
 * "从 trigger 那里长出来"，而不是原地放大缩小（默认的 transform-origin 是
 * 几何中心 50% 50%，视觉上和"从哪个方向冒出来"没有任何关联）。
 * 交叉轴（对齐方向）同理：align 是 start 就贴左/上边，end 就贴右/下边。
 */
export function getTransformOrigin(placement: Placement): string {
  const side = getSide(placement);
  const align = getAlignment(placement);
  const crossAxisOrigin =
    align === "start" ? "0%" : align === "end" ? "100%" : "50%";

  switch (side) {
    case "top":
      return `${crossAxisOrigin} 100%`;
    case "bottom":
      return `${crossAxisOrigin} 0%`;
    case "left":
      return `100% ${crossAxisOrigin}`;
    case "right":
      return `0% ${crossAxisOrigin}`;
  }
}

export interface BuildTooltipMiddlewareOptions {
  sideOffset: number;
  /** 沿交叉轴（对齐方向）的偏移，对应 offset middleware 的 crossAxis */
  alignOffset: number;
  collisionPadding: number;
  arrowElement: () => Element | undefined;
}

/**
 * Tooltip 定位管线的固定配方：
 *
 * 1. offset      — 沿主轴保持 sideOffset 间距，沿交叉轴偏移 alignOffset
 * 2. flip        — 主轴方向放不下就翻到对面（比如上方没空间就翻到下方）
 * 3. shift       — 翻转后仍溢出，就在不换边的前提下贴边平移
 * 4. arrow       — 算出箭头相对 content 的偏移，让它始终指向 trigger 中心
 * 5. hide        — trigger 被自己的滚动容器完全裁掉时标记隐藏
 * 6. containingBlockOffset — 修正祖先元素 transform 等属性导致的 position:fixed
 *    包含块偏移（demo 里通常不出现，放进有大量 transform 的真实项目后才会暴露，
 *    详见该 middleware 自身的注释）。放在最后一位，因为它修正的是"最终要写进
 *    DOM 的坐标"，前面几步都在统一的视口坐标系里计算，不受这个偏移影响。
 *
 * 特意没有用 size 中间件去动态限制 max-height、允许内容超长时滚动——
 * tooltip 不应该出现滚动条（这不是本库的一般性限制，是 tooltip 这个组件
 * 类型本身该有的样子，shadcn/Radix 的 Tooltip 也是这么做的）。宽度上限
 * 靠 TooltipContent.tsx 里静态的 `max-w-xs` class 兜底，让文字自然换行；
 * 高度完全不做限制，交给内容自身的实际高度决定。
 *
 * 这个决定还有一个技术上的原因：早先的实现里试过用 size 中间件动态算
 * max-height 配合 overflow:auto 实现"超长内容可滚动"，结果发现内层元素
 * 一旦同时具备 overflow:auto 和入场动画（zoom-in-95 会给它加一个 transform），
 * 动画播放期间 transform 会让这个元素变成箭头新的包含块（CSS 规范：非 none
 * 的 transform 会创建包含块），箭头戳出内层边缘的那部分又会被算进内层的
 * 可滚动溢出区域——于是出现了"打开瞬间闪一下滚动条，箭头跟着被遮住，
 * 动画播完滚动条消失、箭头才重新出现"这种诡异现象。去掉滚动能力后，
 * 这个耦合从根上就不存在了。
 *
 * 顺序不能随便换：arrow 必须在 shift/flip 之后（要用最终坐标才能算对偏移），
 * containingBlockOffset 必须在所有其他 middleware 之后。
 *
 * 单独抽成函数是为了让 useTooltipContent 保持简洁，也方便针对这套管线单独测试。
 */
export function createTooltipMiddleware(
  options: BuildTooltipMiddlewareOptions,
): Middleware[] {
  return [
    offset({ mainAxis: options.sideOffset, crossAxis: options.alignOffset }),
    flip(),
    shift({ padding: options.collisionPadding }),
    arrow({ element: options.arrowElement, padding: 6 }),
    hide(),
    containingBlockOffset(),
  ];
}

import {
  offset,
  flip,
  shift,
  size,
  hide,
  containingBlockOffset,
  type Middleware,
  type Placement,
} from "~/lib";
import { getViewportBoundary } from "~/lib/positioner/utils/dom";
import type { SelectOptionValue } from "../Select/Select.types";

/** 让浮层宽度和 trigger 对齐——不改坐标，只把 reference 宽度透传给上层当 style.width 用 */
function matchReferenceWidth(): Middleware {
  return {
    name: "matchWidth",
    fn(state) {
      return { data: { width: state.rects.reference.width } };
    },
  };
}

/**
 * "选中项对齐"：让已选中的那一项精确覆盖在 trigger 上方，这是原生 <select>/
 * shadcn Select 重新打开时的经典效果，和 offset+flip+shift 是完全不同的定位
 * 策略，所以单独写一个 middleware，而不是复用 shift/flip。
 *
 * 核心思路是把"面板高度、面板位置、列表滚动量"三者一起算，而不是先假设一个
 * 面板高度、算完位置再回头夹 scrollTop——后者试过两版都有遗留问题（面板和
 * trigger 之间留一截空隙、或者选中项因为 scrollTop 静默越界被推到面板边缘）。
 *
 * 具体算法：选中项"理应"贴着 trigger 垂直居中。以这个位置为基准，
 * 分别看"上方"和"下方"能展开多少：
 * - 上方能展开的量 = min(选中项上方还有多少内容, 视口上方还剩多少空间)
 * - 下方能展开的量 = min(选中项下方还有多少内容, 视口下方还剩多少空间)
 * 面板高度就是"上方展开量 + 选中项自身高度 + 下方展开量"，面板顶部位置和
 * 滚动量都由这两个"展开量"直接算出来，不需要再额外夹取——因为：
 * - 展开量永远不会超过"内容还剩多少"，所以 scrollTop 永远落在合法范围内；
 * - 展开量永远不会超过"视口还剩多少空间"，所以面板永远不会超出视口；
 * - 选中项贴 trigger 是这个算法的出发点，天然成立，不需要单独判断"要不要
 *   贴 trigger"。
 * 对列表两端的项（第一项/最后一项）会自动退化成正确的行为：比如选中第一项，
 * "上方还有多少内容"是 0，上方展开量恒为 0，面板顶部直接落在选中项该在的
 * 位置，不会凭空往上多留空间。
 */
function alignSelectedItem(
  getValue: () => SelectOptionValue | null | undefined,
  getScrollElement: () => HTMLElement | undefined,
): Middleware {
  return {
    name: "itemAlign",
    fn(state) {
      const value = getValue();
      const scrollEl = getScrollElement();
      if (value == null || !scrollEl) return {};

      const itemEl = scrollEl.querySelector<HTMLElement>(
        `[data-value="${CSS.escape(String(value))}"]`,
      );
      if (!itemEl) return {};

      const { rects, strategy } = state;
      const boundary = getViewportBoundary(strategy, 8);

      const itemOffsetTop = itemEl.offsetTop;
      const itemHeight = itemEl.offsetHeight;
      const scrollHeight = scrollEl.scrollHeight;

      // 选中项理想情况下应该落在的位置：和 trigger 垂直居中对齐
      const idealItemTop =
        rects.reference.y + rects.reference.height / 2 - itemHeight / 2;

      const contentAbove = itemOffsetTop; // 选中项上方还有多少内容（px）
      const contentBelow = scrollHeight - itemOffsetTop - itemHeight; // 下方还有多少内容
      const spaceAbove = idealItemTop - boundary.y; // 从理想位置到视口顶部的可用空间
      const spaceBelow =
        boundary.y + boundary.height - (idealItemTop + itemHeight); // 到视口底部的可用空间

      const expandAbove = Math.max(0, Math.min(contentAbove, spaceAbove));
      const expandBelow = Math.max(0, Math.min(contentBelow, spaceBelow));

      const panelTop = idealItemTop - expandAbove;
      // panelHeight 最终会作为 CSS max-height 写到 scrollEl 上。scrollEl
      // 有 border（类名里带 border），而 Tailwind preflight 会把 box-sizing
      // 设为 border-box，此时 max-height 约束的是 border-box 高度；但
      // scrollHeight 只包含内容 + padding、不包含 border。直接拿
      // scrollHeight 当 max-height 会让 clientHeight 比 scrollHeight 少
      // 一个上下 border 的宽度，列表本来刚好能放下时也会多出 1~2px 的
      // overflow，从而被 overflow:auto 画出一条纵向滚动条。这里把 border
      // 宽度补回去，保证 border-box 的 max-height 能完整容纳所有内容。
      const scrollStyle = getComputedStyle(scrollEl);
      const borderTop = Number.parseFloat(scrollStyle.borderTopWidth) || 0;
      const borderBottom =
        Number.parseFloat(scrollStyle.borderBottomWidth) || 0;
      const panelHeight =
        expandAbove + itemHeight + expandBelow + borderTop + borderBottom;
      const scrollTop = contentAbove - expandAbove;

      // scrollTop 是纯 DOM 副作用，不影响坐标计算本身，这里直接设置是安全的。
      scrollEl.scrollTop = scrollTop;

      return {
        x: rects.reference.x,
        y: panelTop,
        data: { aligned: true, panelHeight },
      };
    },
  };
}

export interface BuildSelectMiddlewareOptions {
  /** 当前是否已经有选中值——决定走"选中项对齐"还是普通的贴边下拉 */
  hasValue: boolean;
  selectedValue: () => SelectOptionValue | null | undefined;
  /** 真正会滚动、承载选项列表的那个元素（内层），不是外层的定位容器 */
  scrollElement: () => HTMLElement | undefined;
  placement: Placement;
  collisionPadding: number;
  onAvailableHeightChange: (availableHeight: number) => void;
}

/**
 * Select 面板的定位管线，按"有没有选中值"分两套配方：
 *
 * - **有选中值**：matchWidth → size（算可用高度，配合 overflow:auto 做滚动——
 *   和 Tooltip 不同，下拉列表内容天然可能很长，滚动是合理需求，不是过度设计）
 *   → itemAlign（核心：选中项盖在 trigger 上）→ shift（只处理横向，纵向已经被
 *   itemAlign 自己夹过了）→ containingBlockOffset。
 * - **没有选中值**（占位符状态）：offset → flip → shift → matchWidth → size
 *   → containingBlockOffset，这是标准的"贴边下拉"策略，和 Dropdown 用的是
 *   同一套思路。
 *
 * 顺序不能随便换：size 必须在 shift/itemAlign 之后（要用最终坐标反推可用高度），
 * containingBlockOffset 必须在所有其他 middleware 之后。
 *
 * SelectContent.tsx 里把面板拆成了两层：外层（position:fixed，负责 translate(x,y)
 * 定位）和内层（真正的样式、overflow:auto 滚动、data-state 驱动的进出场动画）。
 * 这不是为了美观随手拆的——同一个元素如果既承担定位用的 transform:translate()，
 * 又承担 zoom-in-95 这类靠 transform:scale() 做的动画，CSS 的 transform 一个
 * 元素只能有一份，动画播放期间会完全接管这个属性，把定位用的 translate(x,y)
 * 整个顶掉，表现为"入场从视口左上角飞入、退场飞向左上角消失"这种明显错误的
 * 观感（这正是这个组件早期实现踩过的一个真实 bug）。scrollElement 传的是
 * 内层（真正会滚动的那个），不是 state.elements.floating（外层，它自己不滚动）。
 */
export function createSelectMiddleware(
  options: BuildSelectMiddlewareOptions,
): Middleware[] {
  if (options.hasValue) {
    return [
      matchReferenceWidth(),
      // 注意这里没有 size 中间件：alignSelectedItem 自己已经把面板高度、
      // 位置、滚动量一起算好了（见它的注释），面板高度是通过
      // middlewareData.itemAlign.panelHeight 传出去的，不需要 size 再用
      // "只看视口空间、不看内容长度"那套算法独立算一遍——两边算出来的值
      // 一旦不一致，CSS max-height 和定位计算假设的面板高度就会对不上，
      // 重新引入"选中项没对齐"这类问题（这正是这次要修的 bug 的根源）。
      alignSelectedItem(options.selectedValue, options.scrollElement),
      shift({
        padding: options.collisionPadding,
        mainAxis: false,
        crossAxis: true,
      }),
      hide(),
      containingBlockOffset(),
    ];
  }

  return [
    offset(4),
    flip(),
    shift({ padding: options.collisionPadding }),
    matchReferenceWidth(),
    size({
      padding: options.collisionPadding,
      apply({ availableHeight }) {
        options.onAvailableHeightChange(availableHeight);
      },
    }),
    hide(),
    containingBlockOffset(),
  ];
}

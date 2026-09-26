import type { Middleware } from "../types";

const TRANSFORM_STYLE_PROPS = [
  "transform",
  "perspective",
  "filter",
  "backdropFilter",
] as const;

function establishesContainingBlock(style: CSSStyleDeclaration): boolean {
  for (const prop of TRANSFORM_STYLE_PROPS) {
    const value = style[prop as keyof CSSStyleDeclaration] as
      | string
      | undefined;
    if (value && value !== "none") return true;
  }
  const willChange = style.willChange || "";
  if (/transform|perspective|filter/.test(willChange)) return true;
  const contain = style.contain || "";
  if (/paint|layout|strict|content/.test(contain)) return true;
  return false;
}

/**
 * 修正因祖先元素设置了 transform / filter / perspective / backdrop-filter /
 * will-change / contain 等 CSS 属性，导致 position:fixed（部分情况下
 * position:absolute 也一样）的包含块被从“视口”重新定义成该祖先自身，
 * 从而让按视口坐标算出来的 x/y 出现系统性偏移的问题。
 *
 * 典型症状：demo 页面里定位完全正常，放进真实项目（尤其是大量使用
 * transform 做拖拽/缩放/面板动画的应用，比如设计工具、看板、画布编辑器）
 * 后，浮层整体固定偏移某个方向，但相对位置关系（比如箭头跟着内容一起偏）
 * 保持不变——这正是“坐标系被祖先重新定义”而不是“定位算法算错了”的特征。
 *
 * 原理：从 floating 元素实际的 DOM 父节点开始向上查找，找到第一个会重新
 * 定义包含块的祖先，用它的 getBoundingClientRect() 反推出这个“意外包含块”
 * 相对视口的偏移量，从最终坐标里减掉，抵消这个偏移。
 *
 * 局限：只处理纯平移类的偏移（比如 translate、面板展开动画、GPU 加速常见的
 * translateZ(0)）。如果那个祖先同时还做了 scale/rotate，这里不会做矩阵换算，
 * 位置依然会有偏差——遇到这种情况，更彻底的办法是让 Portal 挂载点脱离那个
 * 祖先（比如手动把挂载点换成真正不受影响的 document.body，而不是应用内部
 * 某个嵌套很深的容器）。
 *
 * 建议放在 middleware 数组的最后一位——它修正的是“最终要写进 DOM 的坐标”，
 * 前面 offset/flip/shift/size/arrow 等都在统一的视口坐标系里计算，不受影响，
 * 只有真正落地前的这一步需要额外修正。
 */
export function containingBlockOffset(): Middleware {
  return {
    name: "containingBlockOffset",
    fn(state) {
      const floatingEl = state.elements.floating;
      let node: Element | null = floatingEl.parentElement;
      let offsetX = 0;
      let offsetY = 0;
      let foundAncestor: Element | null = null;

      while (node && node !== document.documentElement) {
        const style = getComputedStyle(node);
        if (establishesContainingBlock(style)) {
          const rect = node.getBoundingClientRect();
          offsetX = rect.x;
          offsetY = rect.y;
          foundAncestor = node;
          break;
        }
        node = node.parentElement;
      }

      if (!foundAncestor || (offsetX === 0 && offsetY === 0)) {
        return { data: { ancestor: null } };
      }

      return {
        x: state.x - offsetX,
        y: state.y - offsetY,
        data: { ancestor: foundAncestor, offsetX, offsetY },
      };
    },
  };
}

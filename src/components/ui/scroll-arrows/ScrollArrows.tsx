import type { JSX } from "solid-js";
import { clsx } from "~/utils";
import { useScrollEdges } from "~/hooks/useScrollEdges";
import { ScrollArrowButton } from "./ScrollArrowButton";
import type { ScrollArrowsProps } from "./ScrollArrows.types";

/**
 * 列表/菜单弹层通用的滚动提示：隐藏原生滚动条后，在上/下沿叠加箭头，
 * 提示该方向还有内容；悬停或按住箭头会持续滚动，到边界后箭头自动隐藏。
 * 放在滚动容器(定位祖先)内部即可，需要容器自身是定位元素。
 */
export function ScrollArrows(props: ScrollArrowsProps): JSX.Element {
  const { canScrollUp, canScrollDown } = useScrollEdges(() => props.target());

  return (
    <>
      <ScrollArrowButton
        direction="up"
        visible={canScrollUp()}
        target={props.target}
        class={clsx(props.class, props.upClass)}
      />
      <ScrollArrowButton
        direction="down"
        visible={canScrollDown()}
        target={props.target}
        class={clsx(props.class, props.downClass)}
      />
    </>
  );
}

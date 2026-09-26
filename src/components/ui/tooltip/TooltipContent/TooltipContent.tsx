import { Show, onCleanup, splitProps } from "solid-js";
import { Portal } from "solid-js/web";
import { useTooltipContent } from "./useTooltipContent";
import { TooltipContentContext } from "./TooltipContent.context";
import { getTransformOrigin } from "./TooltipContent.utils";
import { getSide, getAlignment } from "~/lib";
import type { TooltipContentProps } from "./TooltipContent.types";
import { clsx } from "~/utils";

/**
 * Tooltip 的浮层内容。挂 Portal 到 body，用 position:fixed 定位。
 *
 * 内容本身默认是可以 hover 的（不是 pointer-events:none）：鼠标移进 content
 * 会取消待执行的关闭计时器（ctx.keepOpen），移出才真正走 closeDelay 关闭——
 * 这样用户可以把鼠标从 trigger 移动到 tooltip 上选中里面的文字，不会因为
 * "离开 trigger 就关闭" 而选不到。如果你的 tooltip 内容很简单、不需要这个特性，
 * 可以自己在外面包一层设置 pointer-events: none。
 */
export function TooltipContent(props: TooltipContentProps) {
  const {
    ctx,
    pos,
    mounted,
    animationState,
    setArrowElement,
    setContentElement,
    slideStyle,
  } = useTooltipContent(() => props);

  // 只摘出内部要单独处理的几个字段，剩下的（style、data-*、onXxx 等任意原生
  // prop）原样透传给内层元素——和 TooltipTrigger 是同一套约定。
  const [local, rest] = splitProps(props, [
    "side",
    "align",
    "sideOffset",
    "alignOffset",
    "collisionPadding",
    "class",
    "style",
    "children",
  ]);

  return (
    <Show when={mounted()}>
      <Portal>
        {/*
          外层：只负责定位（position:fixed 来自 floatingStyles），不设 overflow，
          也不参与任何动画——它的 transform 已经被占用来做定位（translate(x,y)），
          如果别的动画也要控制 transform 会互相冲突，所以动画都放在下面两层。
        */}
        <div
          ref={(el) => {
            ctx.setFloating(el);
            // Solid 的 ref 回调只在元素创建时触发一次，元素被移除时不会自动
            // 再调用一次传 undefined（这是 React 的约定，不是 Solid 的）。
            // 不手动清空的话，ctx.floating() 会一直指向一个已经从 DOM 里
            // 摘除的"僵尸节点"，直到下次重新打开——期间如果被别处读取
            // （比如 TooltipGroup 抢占时读取矩形），getBoundingClientRect()
            // 对一个 detached 元素规范规定就是返回全 0 的矩形，会造成
            // "滑动效果从左上角(0,0)滑入"这种明显错误的观感。
            onCleanup(() => ctx.setFloating(undefined));
          }}
          data-placement={pos.placement()}
          style={{
            ...pos.floatingStyles(),
            "z-index": 1000,
            opacity: pos.isPositioned() ? 1 : 0,
          }}
        >
          {/*
            滑动层：命中 <TooltipGroup> 抢占时，用来把整个 tooltip（内容+箭头）
            从旧 tooltip 的位置平移过来（FLIP 技术，细节见 useTooltipContent 里
            的 slideOffset 计算）。平时 slideStyle() 是空对象，这一层形同不存在。

            这一层必须夹在"定位层"和"样式/动画层"之间、而不是随便找个地方加：
            它自己也会用到 transform（滑动过程中），如果和定位层共用一个元素会
            跟 translate(x,y) 冲突（原因同上）；但它又不能在样式层（overflow:auto
            那层）的下面/内部，因为箭头的 position:absolute 是故意跳过样式层、
            以"最近的已定位祖先"为基准的（这是修复箭头戳出来触发滚动条那次的
            设计），如果滑动层的 transform 出现在样式层内部，箭头的定位基准就会
            变成滑动层而不是外层——效果上依然正确（滑动层的盒子和外层几乎重合），
            但为了让"定位基准"这件事始终清晰、不产生"技术上是谁、视觉上又是谁"
            的心智负担，把滑动层放在外层和样式层之间、维持这条链路的清晰顺序。
          */}
          <div style={slideStyle()}>
            {/*
              内层：真正的视觉样式 + 进出场动画。
              data-state 跟着 animationState 走（不是 isVisible/mounted）：
              - isVisible 变 false 的瞬间，animationState 立刻跟着变 closed，
                触发退场动画的 class；
              - isVisible 变 true 时，animationState 会晚一帧才变 open，
                留出时间让箭头（TooltipArrow）完成挂载、注册给 arrow middleware、
                算出正确的位置偏移（细节见 useTooltipContent 里的注释）；
              - mounted 会比 animationState 变 closed 更晚才变 false（见下面的
                Presence 逻辑），留出时间让退场动画播完。

              data-[state=open]/data-[state=closed] 这套 class 依赖 tailwindcss-animate
              插件提供的 animate-in/animate-out/fade-in-0/zoom-in-95 等工具类——
              如果你的项目还没装，`npm install tailwindcss-animate` 并在 tailwind
              config 里加上这个 plugin 即可；不想装的话，把这几个 data-[state=...]
              class 换成你自己写的 @keyframes + 对应 class 也一样能工作，
              Presence 逻辑（mounted 延迟卸载）不依赖具体用了哪套动画实现。

              箭头（TooltipArrow）不依赖"作为这个元素的子节点、被动继承这里的
              transform/opacity 动画"这种隐式机制——它自己也从 context 里读到
              同一个 animationState，独立播放一份自己的入场/退场动画，两者由
              同一个信号同步触发。

              data-side/data-align 是从最终生效的 placement 拆出来的，驱动上面
              这几条方向感知的 slide-in class，也顺带给 transform-origin 提供了
              依据——缩放动画的锚点会落在"贴近 trigger 的那条边"上（细节见
              TooltipContent.utils.ts 里的 getTransformOrigin），而不是默认的
              几何中心，这样整个入场效果才会像是"从 trigger 那里长出来"，
              贴近 shadcn 的实际观感。

              特意没有 overflow:auto / max-height，tooltip 不支持内容超长时滚动——
              宽度上限靠下面静态的 max-w-xs class 兜底，让文字自然换行；高度完全
              不做限制。这不只是为了贴近 shadcn/Radix 的行为（它们的 tooltip 也不
              允许滚动），还避免了一个真实踩过的坑：overflow:auto 配合这里的
              zoom-in-95 动画（会给这个元素加一个非 none 的 transform，CSS 规范
              规定这会让它变成子元素新的包含块）曾经导致箭头戳出边缘的那部分被
              误判成"可滚动溢出"，表现为打开瞬间闪一下滚动条、箭头跟着被遮住，
              动画播完滚动条消失、箭头才重新出现。去掉滚动能力后这个耦合从根上
              就不存在了。
            */}
            <div
              ref={setContentElement}
              id={ctx.contentId}
              role="tooltip"
              data-state={animationState()}
              data-side={getSide(pos.placement())}
              data-align={getAlignment(pos.placement()) ?? "center"}
              onMouseEnter={ctx.keepOpen}
              onMouseLeave={ctx.requestClose}
              style={{
                "transform-origin": getTransformOrigin(pos.placement()),
                ...(typeof local.style === "object" ? local.style : undefined),
              }}
              class={clsx(
                "inline-flex w-fit max-w-xs items-center gap-1.5 rounded-md bg-foreground px-3 py-1.5 text-xs text-background",
                "has-data-[slot=kbd]:pr-1.5",
                "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
                "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
                // 方向感知的滑入：不只是原地淡入缩放，还从"贴近 trigger 的反方向"
                // 轻微位移过来——比如 placement 是 top（content 在 trigger 上方），
                // 就从下方 8px（slide-in-from-bottom-2）滑上来，视觉上更像是从
                // trigger 那里"冒出来"，而不是凭空原地出现。只在打开时生效，
                // 贴近 shadcn 的实际行为（它们的退场动画没有对应的滑出位移）。
                "data-[side=top]:slide-in-from-bottom-2",
                "data-[side=bottom]:slide-in-from-top-2",
                "data-[side=left]:slide-in-from-right-2",
                "data-[side=right]:slide-in-from-left-2",
                local.class,
              )}
              {...rest}
            >
              <TooltipContentContext.Provider
                value={{
                  middlewareData: pos.middlewareData,
                  reference: ctx.reference,
                  placement: pos.placement,
                  setArrowElement,
                  animationState,
                }}
              >
                {local.children}
              </TooltipContentContext.Provider>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

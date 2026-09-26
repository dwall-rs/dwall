import { createUniqueId, onMount, Show, splitProps } from "solid-js";
import type { JSX } from "solid-js";
import { clsx } from "~/utils";
import { useTabsContext } from "../Tabs/Tabs.context";
import type { TabsContentProps } from "./TabsContent.types";

export const TabsContent = (props: TabsContentProps) => {
  const [local, others] = splitProps(props, [
    "value",
    "forceMount",
    "class",
    "classList",
    "style",
  ]);
  const ctx = useTabsContext();

  const id = createUniqueId();
  const contentId = `tabs-content-${id}`;

  onMount(() => ctx.setContentId(local.value, contentId));

  const selected = () => ctx.isSelected(local.value);
  const mounted = () => selected() || !!local.forceMount;

  // flex-1 的 display:flex 会覆盖 UA 的 [hidden] 规则,forceMount 隐藏必须显式 display:none
  const style = (): JSX.CSSProperties | undefined => {
    if (selected() || !local.forceMount) return local.style;
    return { ...local.style, display: "none" };
  };

  return (
    <Show when={mounted()}>
      <div
        id={contentId}
        role="tabpanel"
        data-slot="tabs-content"
        aria-labelledby={ctx.getTriggerId(local.value)}
        tabindex="0"
        data-active={selected() ? "" : null}
        class={clsx("flex-1 text-sm outline-none", local.class)}
        classList={local.classList}
        {...others}
        hidden={!selected() && !!local.forceMount}
        style={style()}
      />
    </Show>
  );
};

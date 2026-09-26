import { createUniqueId, onCleanup, onMount, splitProps } from "solid-js";
import { clsx } from "~/utils";
import { useTabsContext } from "../Tabs/Tabs.context";
import { useTabsListContext } from "../TabsList/TabsList.context";
import { callEventHandler } from "../Tabs/call-event-handler";
import { tabsTriggerStyles } from "./TabsTrigger.styles";
import type { TabsTriggerProps } from "./TabsTrigger.types";

export const TabsTrigger = (props: TabsTriggerProps) => {
  const [local, events, others] = splitProps(
    props,
    ["value", "class", "classList", "disabled"],
    ["onClick", "onFocus"],
  );
  const ctx = useTabsContext();
  const listCtx = useTabsListContext();

  const id = createUniqueId();
  const triggerId = `tabs-trigger-${id}`;

  let elementRef: HTMLButtonElement | undefined;

  const disabled = () => !!local.disabled;
  const selected = () => ctx.isSelected(local.value);

  // roving tabindex 跟随焦点高亮:高亮 tab 持 0;
  // 高亮为空或高亮指向的 trigger 不可用(disabled/卸载)时,
  // 首个可用 tab 持 0 作为键盘起点,避免 tablist 键盘不可达
  const tabIndex = () => {
    if (disabled()) return -1;
    const highlighted = ctx.highlightedValue();
    if (highlighted === local.value && ctx.isUsable(local.value)) return 0;
    if (
      (highlighted === undefined || !ctx.isUsable(highlighted)) &&
      ctx.isFirstEnabled(local.value)
    ) {
      return 0;
    }
    return -1;
  };

  onMount(() => {
    ctx.setTriggerId(local.value, triggerId);
    onCleanup(
      ctx.registerTrigger({
        value: local.value,
        disabled,
        element: elementRef!,
      }),
    );
  });

  return (
    <button
      type="button"
      ref={(el) => {
        elementRef = el;
      }}
      role="tab"
      id={triggerId}
      data-slot="tabs-trigger"
      data-active={selected() ? "" : null}
      data-highlighted={ctx.highlightedValue() === local.value ? "" : null}
      aria-selected={selected()}
      aria-controls={ctx.getContentId(local.value)}
      aria-disabled={disabled() ? "true" : undefined}
      tabindex={tabIndex()}
      disabled={local.disabled}
      class={clsx(tabsTriggerStyles, local.class)}
      classList={local.classList}
      {...others}
      onClick={(e) => {
        // base-ui 语义:激活发生在点击(鼠标抬起/Enter/Space)时,
        // mousedown 触发的 focus 不激活
        if (!disabled()) ctx.setValue(local.value);
        callEventHandler(events.onClick, e);
      }}
      onFocus={(e) => {
        // 聚焦只更新高亮(roving focus);activateOnFocus 时聚焦即激活
        if (!disabled()) {
          ctx.setHighlightedValue(local.value);
          if (listCtx.activateOnFocus) ctx.setValue(local.value);
        }
        callEventHandler(events.onFocus, e);
      }}
    />
  );
};

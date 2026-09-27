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

  // roving tabindex follows the focus highlight: the highlighted tab holds 0;
  // when there is no highlight or the highlighted trigger is unusable
  // (disabled/unmounted), the first usable tab holds 0 as the keyboard
  // starting point, keeping the tablist keyboard-reachable
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
        // base-ui semantics: activation happens on click (mouse up / Enter /
        // Space); the focus triggered by mousedown does not activate
        if (!disabled()) ctx.setValue(local.value);
        callEventHandler(events.onClick, e);
      }}
      onFocus={(e) => {
        // Focus only updates the highlight (roving focus); with activateOnFocus, focusing activates
        if (!disabled()) {
          ctx.setHighlightedValue(local.value);
          if (listCtx.activateOnFocus) ctx.setValue(local.value);
        }
        callEventHandler(events.onFocus, e);
      }}
    />
  );
};

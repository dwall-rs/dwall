import { mergeProps, splitProps } from "solid-js";
import type { TabsListProps } from "./TabsList.types";
import { clsx } from "~/utils";
import { tabsListVariants } from "./TabsList.styles";
import { useTabsContext } from "../Tabs/Tabs.context";
import { useTabsKeyboard } from "../Tabs/useTabsKeyboard";
import { callEventHandler } from "../Tabs/call-event-handler";
import { TabsListContext } from "./TabsList.context";

export const TabsList = (props: TabsListProps) => {
  const merged = mergeProps(
    { variant: "secondary", activateOnFocus: false } as const,
    props,
  );
  const [local, events, others] = splitProps(
    merged,
    ["variant", "class", "classList", "activateOnFocus"],
    ["onKeyDown"],
  );

  const ctx = useTabsContext();
  const { handleKeyDown } = useTabsKeyboard(ctx);

  return (
    <TabsListContext.Provider
      value={{ activateOnFocus: local.activateOnFocus }}
    >
      <div
        role="tablist"
        data-slot="tabs-list"
        data-variant={local.variant}
        aria-orientation={
          ctx.orientation() === "vertical" ? "vertical" : undefined
        }
        class={clsx(tabsListVariants({ variant: local.variant }), local.class)}
        classList={local.classList}
        {...others}
        onKeyDown={(e) => {
          handleKeyDown(e);
          callEventHandler(events.onKeyDown, e);
        }}
      />
    </TabsListContext.Provider>
  );
};

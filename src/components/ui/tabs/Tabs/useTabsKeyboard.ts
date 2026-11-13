import { createRovingNavigation } from "~/utils";
import type { TabsContextValue } from "./Tabs.context";

export function useTabsKeyboard(ctx: TabsContextValue) {
  return createRovingNavigation({
    getItems: () => ctx.getTriggers(),
    orientation: ctx.orientation,
    dir: ctx.dir,
    loop: ctx.loop,

    onHighlight: ctx.setHighlightedValue,
  });
}

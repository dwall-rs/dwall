import { mergeProps, splitProps } from "solid-js";
import type { TabsProps } from "./Tabs.types";
import { clsx } from "~/utils";
import { TabsProvider } from "./Tabs.context";

export const Tabs = (props: TabsProps) => {
  const merged = mergeProps(
    {
      orientation: "horizontal",
      loop: true,
      dir: "ltr",
    } as const,
    props,
  );
  const [local, others] = splitProps(merged, [
    "defaultValue",
    "value",
    "onValueChange",
    "orientation",
    "loop",
    "dir",
    "class",
    "classList",
    "children",
  ]);

  const isVertical = () => local.orientation === "vertical";

  return (
    <div
      data-slot="tabs"
      data-orientation={local.orientation}
      // Existing styles rely on group-data-vertical/tabs and
      // group-data-horizontal/tabs, which need the data-vertical /
      // data-horizontal attributes on the root element to take effect
      data-vertical={isVertical() ? "" : null}
      data-horizontal={isVertical() ? null : ""}
      dir={local.dir}
      class={clsx(
        "group/tabs flex gap-2 data-[orientation=horizontal]:flex-col",
        local.class,
      )}
      classList={local.classList}
      {...others}
    >
      <TabsProvider
        defaultValue={local.defaultValue}
        value={local.value}
        onValueChange={local.onValueChange}
        orientation={local.orientation}
        dir={local.dir}
        loop={local.loop}
      >
        {local.children}
      </TabsProvider>
    </div>
  );
};

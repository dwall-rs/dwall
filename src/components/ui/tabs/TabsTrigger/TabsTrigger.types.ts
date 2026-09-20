import type { BaseProps, MouseEventHandler, PolymorphicProps } from "~/types";
import type { TabsValue } from "../Tabs/Tabs.types";

export type TabsTriggerProps = PolymorphicProps<
  "button",
  BaseProps & { value: TabsValue; onClick?: MouseEventHandler<"button"> },
  false
>;

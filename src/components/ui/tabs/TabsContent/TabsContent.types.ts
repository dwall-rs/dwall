import type { BaseProps, PolymorphicProps } from "~/types";
import type { TabsValue } from "../Tabs/Tabs.types";

export type TabsContentProps = PolymorphicProps<
  "div",
  BaseProps & { value: TabsValue; forceMount?: boolean },
  false
>;

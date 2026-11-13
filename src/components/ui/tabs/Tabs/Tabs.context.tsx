import {
  createContext,
  createEffect,
  createSignal,
  type Accessor,
  type ParentProps,
  useContext,
} from "solid-js";
import { createStore } from "solid-js/store";
import type { TabsValue } from "./Tabs.types";

interface TriggerEntry {
  value: TabsValue;

  disabled: () => boolean;
  element: HTMLElement;
}

export interface TabsContextValue {
  value: Accessor<TabsValue | undefined>;
  setValue: (value: TabsValue) => void;
  isSelected: (value: TabsValue) => boolean;
  orientation: Accessor<"horizontal" | "vertical">;
  dir: Accessor<"ltr" | "rtl" | "auto">;
  loop: Accessor<boolean>;

  highlightedValue: Accessor<TabsValue | undefined>;
  setHighlightedValue: (value: TabsValue) => void;

  registerTrigger: (entry: TriggerEntry) => () => void;

  getTriggers: () => TriggerEntry[];

  isFirstEnabled: (value: TabsValue) => boolean;

  isUsable: (value: TabsValue) => boolean;

  setTriggerId: (value: TabsValue, id: string) => void;
  getTriggerId: (value: TabsValue) => string | undefined;
  setContentId: (value: TabsValue, id: string) => void;
  getContentId: (value: TabsValue) => string | undefined;
}

const TabsContext = createContext<TabsContextValue>();

export function TabsProvider(
  props: ParentProps & {
    defaultValue?: TabsValue;
    value?: TabsValue;
    onValueChange?: (value: TabsValue) => void;
    orientation: "horizontal" | "vertical";
    dir: "ltr" | "rtl" | "auto";
    loop: boolean;
  },
) {
  const [internalValue, setInternalValue] = createSignal<TabsValue | undefined>(
    props.defaultValue,
  );
  const [highlightedValue, setHighlightedValue] = createSignal<
    TabsValue | undefined
  >();
  const [triggerOrder, setTriggerOrder] = createSignal<TriggerEntry[]>([]);
  const [ids, setIds] = createStore<{
    trigger: Record<string, string>;
    content: Record<string, string>;
  }>({ trigger: {}, content: {} });

  const value = (): TabsValue | undefined => props.value ?? internalValue();

  const setValue = (next: TabsValue) => {
    if (props.value !== undefined) {
      props.onValueChange?.(next);
      return;
    }
    setInternalValue(next);
    props.onValueChange?.(next);
  };

  const isSelected = (v: TabsValue) => value() === v;

  createEffect(() => {
    const current = value();
    if (current === undefined) return;
    const active = document.activeElement as HTMLElement | null;
    if (active?.getAttribute("role") !== "tab") {
      setHighlightedValue(current);
    }
  });

  const registerTrigger = (entry: TriggerEntry) => {
    setTriggerOrder((prev) => [...prev, entry]);
    return () =>
      setTriggerOrder((prev) => prev.filter((t) => t.value !== entry.value));
  };

  const isFirstEnabled = (v: TabsValue) => {
    const first = triggerOrder().find((t) => !t.disabled());
    return first?.value === v;
  };

  const isUsable = (v: TabsValue) =>
    triggerOrder().some((t) => t.value === v && !t.disabled());

  const setTriggerId = (v: TabsValue, id: string) =>
    setIds("trigger", String(v), id);
  const getTriggerId = (v: TabsValue) => ids.trigger[String(v)];
  const setContentId = (v: TabsValue, id: string) =>
    setIds("content", String(v), id);
  const getContentId = (v: TabsValue) => ids.content[String(v)];

  const contextValue: TabsContextValue = {
    value,
    setValue,
    isSelected,
    orientation: () => props.orientation,
    dir: () => props.dir,
    loop: () => props.loop,
    highlightedValue,
    setHighlightedValue,
    registerTrigger,
    getTriggers: () => triggerOrder(),
    isFirstEnabled,
    isUsable,
    setTriggerId,
    getTriggerId,
    setContentId,
    getContentId,
  };

  return (
    <TabsContext.Provider value={contextValue}>
      {props.children}
    </TabsContext.Provider>
  );
}

export const useTabsContext = () => {
  const context = useContext(TabsContext);
  if (!context) throw new Error("useTabsContext 必须用在 <Tabs> 内部");
  return context;
};

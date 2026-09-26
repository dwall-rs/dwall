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
  /** accessor,支持运行时 disabled 翻转 */
  disabled: () => boolean;
  element: HTMLElement;
}

export interface TabsContextValue {
  /** 当前激活值(受控优先,否则内部状态) */
  value: Accessor<TabsValue | undefined>;
  setValue: (value: TabsValue) => void;
  isSelected: (value: TabsValue) => boolean;
  orientation: Accessor<"horizontal" | "vertical">;
  dir: Accessor<"ltr" | "rtl" | "auto">;
  loop: Accessor<boolean>;
  /**
   * 当前焦点高亮 tab,与激活值分离:
   * 默认(activateOnFocus=false)聚焦/键盘导航只更新高亮,不激活;
   * 激活只发生在点击(click)时。与 base-ui 的 highlighted/active 语义一致。
   */
  highlightedValue: Accessor<TabsValue | undefined>;
  setHighlightedValue: (value: TabsValue) => void;
  /** 按挂载顺序注册 trigger,返回注销函数;仅用于渲染期推导与键盘导航 */
  registerTrigger: (entry: TriggerEntry) => () => void;
  /** 已注册的 trigger 列表(挂载顺序) */
  getTriggers: () => TriggerEntry[];
  /** 无高亮时,该 value 是否为第一个可用 trigger(键盘 tabindex 起点) */
  isFirstEnabled: (value: TabsValue) => boolean;
  /** 该 value 对应的 trigger 当前是否可用(已注册且未 disabled) */
  isUsable: (value: TabsValue) => boolean;
  /** value -> triggerId / contentId 交叉注册,供 aria-controls / aria-labelledby 使用 */
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

  // 受控优先:props.value 存在时读外部值,否则读内部状态
  const value = (): TabsValue | undefined => props.value ?? internalValue();

  const setValue = (next: TabsValue) => {
    // 受控模式:只通知外部,不改本地状态
    if (props.value !== undefined) {
      props.onValueChange?.(next);
      return;
    }
    setInternalValue(next);
    props.onValueChange?.(next);
  };

  const isSelected = (v: TabsValue) => value() === v;

  // 对齐 base-ui:激活值变化时,若焦点不在 tab 上,将高亮同步到激活 tab
  // (键盘 roving focus 过程中不覆盖,保持相对当前位置导航)
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
  if (!context)
    throw new Error("useTabsContext must be used within a <Tabs> component");
  return context;
};

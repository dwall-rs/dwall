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
  /** accessor, supports runtime disabled flips */
  disabled: () => boolean;
  element: HTMLElement;
}

export interface TabsContextValue {
  /** Current activated value (controlled takes precedence, else internal state) */
  value: Accessor<TabsValue | undefined>;
  setValue: (value: TabsValue) => void;
  isSelected: (value: TabsValue) => boolean;
  orientation: Accessor<"horizontal" | "vertical">;
  dir: Accessor<"ltr" | "rtl" | "auto">;
  loop: Accessor<boolean>;
  /**
   * The currently focused/highlighted tab, separate from the activated value:
   * by default (activateOnFocus=false) focus/keyboard navigation only moves
   * the highlight, never activates; activation happens only on click.
   * Matches base-ui's highlighted/active semantics.
   */
  highlightedValue: Accessor<TabsValue | undefined>;
  setHighlightedValue: (value: TabsValue) => void;
  /** Register a trigger in mount order, returning a deregistration function; used only for render-time derivation and keyboard navigation */
  registerTrigger: (entry: TriggerEntry) => () => void;
  /** Registered trigger list (mount order) */
  getTriggers: () => TriggerEntry[];
  /** With no highlight, whether this value is the first usable trigger (the keyboard tabindex starting point) */
  isFirstEnabled: (value: TabsValue) => boolean;
  /** Whether the trigger for this value is currently usable (registered and not disabled) */
  isUsable: (value: TabsValue) => boolean;
  /** Cross-registration of value -> triggerId / contentId, for aria-controls / aria-labelledby */
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

  // Controlled takes precedence: read the external value when props.value is present, otherwise internal state
  const value = (): TabsValue | undefined => props.value ?? internalValue();

  const setValue = (next: TabsValue) => {
    // Controlled mode: only notify the outside, never touch local state
    if (props.value !== undefined) {
      props.onValueChange?.(next);
      return;
    }
    setInternalValue(next);
    props.onValueChange?.(next);
  };

  const isSelected = (v: TabsValue) => value() === v;

  // Align with base-ui: when the activated value changes while focus is not on
  // a tab, sync the highlight to the active tab (not overridden during keyboard
  // roving focus, preserving navigation relative to the current position)
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

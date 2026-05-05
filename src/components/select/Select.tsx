import { createSignal, createEffect, For, Show, onCleanup } from "solid-js";
import { clsx } from "~/utils";
import { Check, ChevronRight } from "lucide-solid";

export interface SelectOption<T> {
  value: T;
  label: string;
  disabled?: boolean;
}

interface SelectProps<T extends string> {
  options: SelectOption<T>[];
  value: T;
  onChange: (value: T) => void;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
}

export function Select<T extends string>(props: SelectProps<T>) {
  let triggerRef: HTMLButtonElement | undefined;
  let listRef: HTMLDivElement | undefined;

  const [open, setOpen] = createSignal(false);
  const [visible, setVisible] = createSignal(false);

  const selectedLabel = () =>
    props.options.find((o) => o.value === props.value)?.label;

  const openMenu = () => {
    setVisible(true);
    // Two rAF to allow DOM paint before triggering transition
    requestAnimationFrame(() => requestAnimationFrame(() => setOpen(true)));
  };

  const closeMenu = () => {
    setOpen(false);
  };

  const toggle = () => {
    if (props.disabled) return;
    visible() ? closeMenu() : openMenu();
  };

  const handleTransitionEnd = () => {
    if (!open()) setVisible(false);
  };

  const select = (value: string, disabled?: boolean) => {
    if (disabled) return;
    props.onChange(value as T);
    closeMenu();
  };

  const handleOutsideClick = (e: MouseEvent) => {
    if (
      !triggerRef?.contains(e.target as Node) &&
      !listRef?.contains(e.target as Node)
    ) {
      closeMenu();
    }
  };

  createEffect(() => {
    if (visible()) {
      document.addEventListener("mousedown", handleOutsideClick);
    } else {
      document.removeEventListener("mousedown", handleOutsideClick);
    }
  });

  onCleanup(() =>
    document.removeEventListener("mousedown", handleOutsideClick),
  );

  const handleKeyDown = (e: KeyboardEvent) => {
    if (props.disabled) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggle();
    } else if (e.key === "Escape") {
      closeMenu();
      triggerRef?.focus();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const enabled = props.options.filter((o) => !o.disabled);
      const idx = enabled.findIndex((o) => o.value === props.value);
      const next =
        e.key === "ArrowDown"
          ? (idx + 1) % enabled.length
          : (idx - 1 + enabled.length) % enabled.length;
      props.onChange(enabled[next]?.value);
      if (!visible()) openMenu();
    }
  };

  return (
    <div class={clsx("relative w-full", props.class)}>
      {/* Trigger */}
      <button
        ref={triggerRef}
        type="button"
        role="combobox"
        aria-expanded={open()}
        aria-haspopup="listbox"
        disabled={props.disabled}
        onKeyDown={handleKeyDown}
        onClick={toggle}
        class={clsx(
          "flex h-9 w-full items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm shadow-xs",
          "transition-[color,box-shadow] outline-none",
          "border-neutral-200 dark:border-white/15 bg-transparent",
          "focus-visible:border-neutral-400 dark:focus-visible:border-neutral-500 focus-visible:ring-3 focus-visible:ring-neutral-400/50 dark:focus-visible:ring-neutral-500/50",
          "disabled:pointer-events-none disabled:cursor-not-allowed disabled:select-none disabled:bg-neutral-200/50 disabled:opacity-50",
          "dark:bg-white/4.5 dark:disabled:bg-white/12",
          "hover:bg-neutral-100 dark:hover:bg-neutral-700",
          "active:bg-neutral-200 dark:active:bg-neutral-600",
          open() &&
            "ring-neutral-400/50 dark:ring-neutral-500/50 ring-[3px] border-neutral-400 dark:border-neutral-500",
        )}
      >
        <span
          class={
            selectedLabel() ? "" : "text-neutral-500 dark:text-neutral-400"
          }
        >
          {selectedLabel() ?? props.placeholder ?? "Select…"}
        </span>
        <ChevronRight
          width="16"
          height="16"
          aria-hidden="true"
          class={clsx(
            "text-muted-foreground shrink-0 transition-transform duration-200",
            open() ? "rotate-90" : "",
          )}
        />
      </button>

      {/* Dropdown list */}
      <Show when={visible()}>
        <div
          ref={listRef}
          role="listbox"
          onTransitionEnd={handleTransitionEnd}
          class={clsx(
            "absolute left-0 top-[calc(100%+4px)] z-50 w-full",
            "rounded-md border border-neutral-200 dark:border-white/10 bg-white dark:bg-neutral-900 text-neutral-950 dark:text-neutral-50 shadow-md",
            "overflow-hidden",
            "transition-all duration-200 ease-out origin-top",
            open()
              ? "opacity-100 scale-y-100 translate-y-0"
              : "opacity-0 scale-y-95 -translate-y-1 pointer-events-none",
          )}
        >
          <div class="p-1">
            <For each={props.options}>
              {(option) => (
                <div
                  role="option"
                  tabIndex={0}
                  aria-selected={props.value === option.value}
                  aria-disabled={option.disabled}
                  onClick={() => select(option.value, option.disabled)}
                  class={clsx(
                    "relative flex select-none items-center rounded-sm py-1.5 pl-3 text-sm outline-none",
                    "transition-colors duration-100",
                    option.disabled
                      ? "pointer-events-none opacity-50"
                      : "hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-900 dark:hover:text-neutral-50 active:bg-neutral-200 dark:active:bg-neutral-700 active:text-neutral-800 dark:active:text-neutral-100",
                    props.value === option.value
                      ? "bg-neutral-100/40 dark:bg-neutral-800/40 font-medium"
                      : "",
                  )}
                >
                  {option.label}

                  <Show when={props.value === option.value}>
                    <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
                      <Check />
                    </span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}

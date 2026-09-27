import { Show, splitProps, onCleanup } from "solid-js";
import { useSelectItem } from "./useSelectItem";
import type { SelectItemProps } from "./SelectItem.types";
import { clsx } from "~/utils";
import { Check } from "lucide-solid";

export function SelectItem(props: SelectItemProps) {
  const { isSelected, isActive, select, onMouseEnter } = useSelectItem(
    () => props,
  );

  const [local, rest] = splitProps(props, [
    "value",
    "label",
    "disabled",
    "class",
    "children",
  ]);

  // click/mouseenter go through addEventListener (not JSX onClick), the same
  // convention as SelectTrigger: no onXxx prop names are consumed, and an
  // onClick passed by the caller is never overridden.
  const attachListeners = (el: HTMLDivElement) => {
    const handleClick = () => select();
    const handleMouseEnter = () => onMouseEnter();
    el.addEventListener("click", handleClick);
    el.addEventListener("mouseenter", handleMouseEnter);
    onCleanup(() => {
      el.removeEventListener("click", handleClick);
      el.removeEventListener("mouseenter", handleMouseEnter);
    });
  };

  return (
    <div
      ref={attachListeners}
      role="option"
      tabIndex={isSelected() ? 0 : -1}
      data-value={String(local.value)}
      aria-selected={isSelected()}
      aria-disabled={local.disabled}
      class={clsx(
        "relative flex cursor-pointer select-none items-center rounded-sm py-1.5 pl-2 pr-6 text-sm outline-none",
        isActive() && "bg-neutral-100 dark:bg-neutral-800",
        local.disabled && "pointer-events-none opacity-50",
        local.class,
      )}
      {...rest}
    >
      {local.children ?? local.label ?? local.value}
      <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
        <Show when={isSelected()}>
          <Check size={14} />
        </Show>
      </span>
    </div>
  );
}

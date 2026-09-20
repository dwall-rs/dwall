import { Show, splitProps, onCleanup } from "solid-js";
import { useSelectItem } from "./useSelectItem";
import type { SelectItemProps } from "./SelectItem.types";

function cn(...classes: Array<string | false | undefined | null>): string {
  return classes.filter(Boolean).join(" ");
}

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

  // click/mouseenter 走 addEventListener（不是 JSX onClick），和 SelectTrigger
  // 是同一套约定，不占用 onXxx prop 名，调用方自己传的 onClick 不会被覆盖。
  const attachListeners = (el: HTMLLIElement) => {
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
    <li
      ref={attachListeners}
      role="option"
      tabIndex={-1}
      data-value={local.value}
      aria-selected={isSelected()}
      aria-disabled={local.disabled}
      class={cn(
        "relative flex cursor-pointer select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none",
        isActive() && "bg-neutral-100 dark:bg-neutral-800",
        local.disabled && "pointer-events-none opacity-50",
        local.class,
      )}
      {...rest}
    >
      <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
        <Show when={isSelected()}>
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <title>selected</title>
            <path
              d="M20 6L9 17l-5-5"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </Show>
      </span>
      {local.children ?? local.label ?? local.value}
    </li>
  );
}

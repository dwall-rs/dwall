// Responsibility: one settings section — heading plus a single shared surface.
// The surface belongs to the group, so rows divide with rules instead of each becoming
// an identical card.
import type { JSXElement } from "solid-js";
import { clsx } from "@/utils";

interface Props {
  icon: JSXElement;
  title: string;
  id?: string;
  bare?: boolean;
  children: JSXElement;
}

export function SettingsGroup(props: Props) {
  return (
    <section id={props.id} class="mb-10 scroll-mt-6">
      <div class="mb-3 flex items-center gap-2.5">
        <span class="grid size-7 shrink-0 place-items-center rounded-[9px] bg-accent text-accent-foreground">
          {props.icon}
        </span>
        <h2 class="font-display text-[18px] font-semibold tracking-[-0.015em]">
          {props.title}
        </h2>
      </div>
      <div
        class={clsx(
          !props.bare &&
            "overflow-hidden rounded-[16px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.03)]",
        )}
      >
        {props.children}
      </div>
    </section>
  );
}

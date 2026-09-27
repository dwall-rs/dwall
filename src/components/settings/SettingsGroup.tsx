// Responsibility: Settings group container — icon + title + card slot; pure presentation, staggered entrance.
//       Note: do not destructure props (in Solid, destructuring loses reactivity, so language switches etc. cannot update).

import type { JSXElement } from "solid-js";

interface Props {
  icon: JSXElement;
  title: string;
  delay?: number;
  children: JSXElement;
}

export function SettingsGroup(props: Props) {
  return (
    <div
      class="mb-7.5 animate-rise"
      style={{ "animation-delay": `${props.delay ?? 0}ms` }}
    >
      <h3 class="mb-3 flex items-center gap-2.25 font-display text-[11px] font-bold uppercase tracking-[1.6px] text-muted-foreground">
        <span class="grid size-5.5 place-items-center rounded-[7px] bg-accent text-primary">
          {props.icon}
        </span>
        {props.title}
      </h3>
      {props.children}
    </div>
  );
}

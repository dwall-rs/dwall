// Responsibility: the single page-title treatment used by every top-level surface.
import type { JSXElement } from "solid-js";

interface Props {
  title: string;
  subtitle?: string;
  children?: JSXElement;
}

export function PageHeader(props: Props) {
  return (
    <header class="mb-6 flex flex-wrap items-end justify-between gap-x-6 gap-y-4 border-b border-border pb-5">
      <div class="min-w-0">
        <h1 class="font-display text-[30px] font-semibold leading-[1.1] tracking-[-0.025em] text-foreground">
          {props.title}
        </h1>
        <p class="mt-2 max-w-[62ch] text-[13px] leading-relaxed text-muted-foreground">
          {props.subtitle}
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">{props.children}</div>
    </header>
  );
}

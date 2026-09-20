// 职责：设置分组容器——图标 + 标题 + 卡片槽；纯展示，stagger 入场。

import type { JSXElement } from "solid-js";

interface Props {
  icon: JSXElement;
  title: string;
  delay?: number;
  children: JSXElement;
}

export function SettingsGroup({ icon, title, delay = 0, children }: Props) {
  return (
    <div
      class="mb-7.5 animate-rise"
      style={{ "animation-delay": `${delay}ms` }}
    >
      <h3 class="mb-3 flex items-center gap-2.25 font-display text-[11px] font-bold uppercase tracking-[1.6px] text-muted-foreground">
        <span class="grid size-5.5 place-items-center rounded-[7px] bg-accent text-primary">
          {icon}
        </span>
        {title}
      </h3>
      {children}
    </div>
  );
}

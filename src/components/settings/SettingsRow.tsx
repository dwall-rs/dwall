// 职责：设置行容器；非堆叠布局加 flex-wrap，窄幅时控件自动换行到标签下方。

import type { JSXElement } from "solid-js";
import { clsx } from "@/utils";
import { Check } from "lucide-solid";

interface Props {
  label: string;
  desc?: JSXElement;
  okShow?: boolean;
  control: JSXElement;
  stacked?: boolean;
}

export function SettingsRow({ label, desc, okShow, control, stacked }: Props) {
  return (
    <div
      class={clsx(
        "flex gap-4.5 border-b border-border p-[16px_18px] transition-colors last:border-b-0 hover:bg-foreground/2",
        stacked ? "flex-col items-stretch" : "flex-wrap gap-y-3",
      )}
    >
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2 font-display text-[14.5px] font-bold">
          {label}
          <Check
            class={clsx(
              "size-3.5 text-success transition-all duration-300",
              okShow ? "scale-100 opacity-100" : "scale-50 opacity-0",
            )}
          />
        </div>
        {desc && (
          <div class="mt-1.25 max-w-110 text-[12px] leading-relaxed text-muted-foreground">
            {desc}
          </div>
        )}
      </div>
      {!stacked && (
        <div class="flex shrink-0 items-center gap-2">{control}</div>
      )}
      {stacked && <div class="mt-1">{control}</div>}
    </div>
  );
}

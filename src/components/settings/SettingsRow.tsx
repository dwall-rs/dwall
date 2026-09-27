// Responsibility: Settings row — built on Field, providing label/description semantics with horizontal/vertical layout.
//       Note: do not destructure props (in Solid, destructuring loses reactivity, so language switches etc. cannot update).
import type { JSXElement } from "solid-js";
import { Show } from "solid-js";
import { Check } from "lucide-solid";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldTitle,
} from "@/components/ui/field";
import { clsx } from "@/utils";

interface Props {
  label: string;
  desc?: JSXElement;
  okShow?: boolean;
  control: JSXElement;
  stacked?: boolean;
}

export function SettingsRow(props: Props) {
  return (
    <Field
      orientation={props.stacked ? "vertical" : "horizontal"}
      class={clsx(
        "border-b border-border p-[16px_18px] transition-colors last:border-b-0 hover:bg-foreground/2",
        !props.stacked && "flex-wrap gap-y-3",
      )}
    >
      <FieldContent class="gap-1">
        <FieldTitle class="text-[14.5px] font-bold!">
          {props.label}
          <Check
            class={clsx(
              "size-3.5 text-success transition-all duration-300",
              props.okShow ? "scale-100 opacity-100" : "scale-50 opacity-0",
            )}
          />
        </FieldTitle>
        <Show when={props.desc}>
          <FieldDescription class="max-w-110 text-[12px] leading-relaxed">
            {props.desc}
          </FieldDescription>
        </Show>
      </FieldContent>
      <div class="shrink-0">{props.control}</div>
    </Field>
  );
}

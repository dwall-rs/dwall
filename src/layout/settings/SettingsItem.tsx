import { mergeProps, Show, type JSXElement } from "solid-js";

import {
  Field,
  FieldContent,
  FieldDescription,
  FieldLabel,
} from "~/components/field";

interface SettingsItemProps {
  label: string;
  orientation?: "horizontal" | "vertical";
  children: JSXElement;
  help?: JSXElement;
  for?: string;
  extra?: JSXElement;
}

const SettingsItem = (props: SettingsItemProps) => {
  const merged = mergeProps({ orientation: "horizontal" } as const, props);

  return (
    <Field orientation={merged.orientation}>
      <Show
        when={merged.orientation === "horizontal"}
        fallback={<FieldLabel for={merged.for}>{props.label}</FieldLabel>}
      >
        <Show
          when={merged.help}
          fallback={<FieldLabel for={merged.for}>{props.label}</FieldLabel>}
        >
          <FieldContent>
            <FieldLabel for={merged.for}>{props.label}</FieldLabel>
            <FieldDescription class="text-xs font-light">
              {merged.help}
            </FieldDescription>
          </FieldContent>
        </Show>
      </Show>
      {merged.children}
      <Show when={merged.orientation === "vertical" && merged.help}>
        <FieldDescription class="text-xs font-light">
          {merged.help}
        </FieldDescription>
      </Show>
    </Field>
  );
};

export default SettingsItem;

export interface SelectSeparatorProps {
  class?: string;
}

export function SelectSeparator(props: SelectSeparatorProps) {
  return (
    <div
      class={`-mx-1 my-1 h-px bg-neutral-200 dark:bg-neutral-800 ${props.class ?? ""}`}
    />
  );
}

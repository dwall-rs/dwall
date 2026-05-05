import { children, Show, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { FieldErrorProps } from "./FieldError.types";

export const FieldError = (props: FieldErrorProps) => {
  const [local, others] = splitProps(props, [
    "errors",
    "class",
    "classList",
    "children",
  ]);

  const resolved = children(() => {
    if (local.children) return local.children;

    if (!local.errors?.length) return null;

    const uniqueErrors = [
      ...new Map(local.errors.map((error) => [error?.message, error])).values(),
    ];

    if (uniqueErrors?.length === 1) {
      return uniqueErrors[0]?.message;
    }

    return (
      <ul class="ml-4 flex list-disc flex-col gap-1">
        {uniqueErrors.map(
          (error) => error?.message && <li>{error.message}</li>,
        )}
      </ul>
    );
  });

  return (
    <Show when={resolved()}>
      <div
        role="alert"
        data-slot="field-error"
        class={clsx(
          "text-sm font-normal text-red-600 dark:text-red-400",
          local.class,
        )}
        {...others}
      >
        {resolved()}
      </div>
    </Show>
  );
};

import type { JSXElement } from "solid-js";
import type { PromiseData, PromiseT } from "../Toast/Toast.types";
import type { CreateToastInput } from "./store";

export interface PromiseToastPorts {
  show(data: CreateToastInput): string;
  dismiss(id: string): void;
}

type PromiseOutcome = "success" | "error";

async function resolveValue(resolver: unknown, value: unknown) {
  return typeof resolver === "function"
    ? await (resolver as (input: unknown) => JSXElement | string)(value)
    : resolver;
}

export function createPromiseToast<Data = any>(
  promise: PromiseT<Data>,
  data: PromiseData<Data> | undefined,
  ports: PromiseToastPorts,
): string | undefined {
  if (!data) return undefined;

  let id: string | undefined;
  if (data.loading !== undefined) {
    id = ports.show({
      ...data,
      type: "loading",
      message: data.loading,
    } as CreateToastInput);
  }

  const settle = async (type: PromiseOutcome, outcome: unknown) => {
    const message = await resolveValue(
      type === "success" ? data.success : data.error,
      outcome,
    );

    if (message === undefined) {
      if (id !== undefined) ports.dismiss(id);
      return;
    }

    ports.show({
      id,
      type,
      message: message as JSXElement,
      description: (await resolveValue(
        data.description,
        outcome,
      )) as JSXElement,
    });
  };

  Promise.resolve(typeof promise === "function" ? promise() : promise)
    .then((result) => settle("success", result))
    .catch((error) => settle("error", error))
    .finally(() => {
      data.finally?.();
    });

  return id;
}

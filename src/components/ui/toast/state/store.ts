import { createUniqueId, type JSXElement } from "solid-js";
import { createStore, produce } from "solid-js/store";
import type {
  ExternalToast,
  PromiseT,
  ToastT,
  ToastTypes,
} from "../Toast/Toast.types";

const [toasts, setToasts] = createStore<ToastT[]>([]);

export function useSonner() {
  return { toasts };
}

export function getToasts() {
  return toasts;
}

export function getHistory() {
  return toasts;
}

export type CreateToastInput = ExternalToast & {
  message?: JSXElement;
  type?: ToastTypes;
  jsx?: JSXElement;
  promise?: PromiseT;
};

export function createToast(data: CreateToastInput): string {
  const id = data.id ?? createUniqueId();

  setToasts(
    produce((list) => {
      const existing = list.find((toast) => toast.id === id);
      const next: ToastT = {
        ...data,
        id,
        title: data.message,
        dismissible: data.dismissible ?? true,
      } as ToastT;

      if (existing) {
        Object.assign(existing, next, {
          title: data.message ?? existing.title,
        });
      } else {
        list.unshift(next);
      }
    }),
  );

  return id;
}

export function dismissToast(id?: string) {
  setToasts(
    produce((list) => {
      if (id !== undefined) {
        const toast = list.find((item) => item.id === id);
        if (toast) toast.delete = true;
        return;
      }
      for (const toast of list) toast.delete = true;
    }),
  );

  return id;
}

export function removeToast(id: string) {
  setToasts(
    produce((list) => {
      const index = list.findIndex((toast) => toast.id === id);
      if (index !== -1) list.splice(index, 1);
    }),
  );
}

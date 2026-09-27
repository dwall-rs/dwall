import { createUniqueId, type JSXElement } from "solid-js";
import { createStore, produce } from "solid-js/store";
import type {
  ExternalToast,
  PromiseData,
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

function createToast(
  data: ExternalToast & {
    message?: JSXElement;
    type?: ToastTypes;
    jsx?: JSXElement;
    promise?: PromiseT;
  },
): string {
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

function dismissToast(id?: string) {
  if (id !== undefined) {
    setToasts(
      produce((list) => {
        const toast = list.find((item) => item.id === id);
        if (toast) toast.delete = true;
      }),
    );
  } else {
    setToasts(
      produce((list) => {
        for (const toast of list) toast.delete = true;
      }),
    );
  }
  return id;
}

function removeToast(id: string) {
  setToasts(
    produce((list) => {
      const index = list.findIndex((toast) => toast.id === id);
      if (index !== -1) list.splice(index, 1);
    }),
  );
}

const toastFunction = (message: JSXElement, data?: ExternalToast) =>
  createToast({ ...data, message });

export const toast = Object.assign(toastFunction, {
  success: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, type: "success", message }),
  info: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, type: "info", message }),
  warning: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, type: "warning", message }),
  error: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, type: "error", message }),
  loading: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, type: "loading", message }),
  message: (message: JSXElement, data?: ExternalToast) =>
    createToast({ ...data, message }),
  custom: (jsx: (id: string) => JSXElement, data?: ExternalToast) => {
    const id = data?.id ?? createUniqueId();
    return createToast({ ...data, id, jsx: jsx(id) });
  },
  dismiss: dismissToast,
  promise: <Data = any>(promise: PromiseT<Data>, data?: PromiseData<Data>) => {
    if (!data) return;

    let id: string | undefined;
    if (data.loading !== undefined) {
      id = createToast({
        ...data,
        type: "loading",
        message: data.loading,
      } as any);
    }

    Promise.resolve(typeof promise === "function" ? promise() : promise)
      .then(async (result) => {
        const message =
          typeof data.success === "function"
            ? await data.success(result)
            : data.success;

        if (message !== undefined) {
          createToast({
            id,
            type: "success",
            message: message as JSXElement,
            description:
              typeof data.description === "function"
                ? await data.description(result)
                : data.description,
          } as any);
        } else if (id !== undefined) {
          dismissToast(id);
        }
      })
      .catch(async (error) => {
        const message =
          typeof data.error === "function"
            ? await data.error(error)
            : data.error;

        if (message !== undefined) {
          createToast({
            id,
            type: "error",
            message: message as JSXElement,
            description:
              typeof data.description === "function"
                ? await data.description(error)
                : data.description,
          } as any);
        } else if (id !== undefined) {
          dismissToast(id);
        }
      })
      .finally(() => {
        data.finally?.();
      });

    return id as string;
  },
  getToasts,
  getHistory,
  remove: removeToast,
});

export { dismissToast, removeToast };

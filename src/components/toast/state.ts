/**
 * state.ts
 * Pure observable store — no framework dependency.
 * Responsible for: creating, updating, dismissing toasts, and notifying subscribers.
 */
import { createUniqueId, type JSXElement } from "solid-js";
import type {
  ExternalToast,
  PromiseData,
  PromiseIExtendedResult,
  PromiseT,
  ToastT,
  ToastToDismiss,
  ToastTypes,
} from "./types";

const isValidElement = (src: object): boolean => {
  return !("id" in src && "message" in src);
};

// ─── Observer (pub/sub) ────────────────────────────────────────────────────────
class Observer {
  subscribers: Array<(toast: ToastT | ToastToDismiss) => void> = [];
  toasts: Array<ToastT | ToastToDismiss> = [];
  dismissedToasts: Set<string> = new Set();

  subscribe = (subscriber: (toast: ToastT | ToastToDismiss) => void) => {
    this.subscribers.push(subscriber);
    return () => {
      const index = this.subscribers.indexOf(subscriber);
      this.subscribers.splice(index, 1);
    };
  };

  publish = (data: ToastT) => {
    for (const subscriber of this.subscribers) {
      subscriber(data);
    }
  };

  addToast = (data: ToastT) => {
    this.publish(data);
    this.toasts = [...this.toasts, data];
  };

  create = (
    data: ExternalToast & {
      message?: JSXElement;
      type?: ToastTypes;
      promise?: PromiseT;
      jsx?: (() => JSXElement) | JSXElement;
    },
  ) => {
    const { message, ...rest } = data;
    const id =
      typeof data?.id === "number" || (data.id as string)?.length > 0
        ? data.id!
        : createUniqueId();

    const alreadyExists = this.toasts.find((t) => t.id === id);
    const dismissible =
      data.dismissible === undefined ? true : data.dismissible;

    if (this.dismissedToasts.has(id)) this.dismissedToasts.delete(id);

    if (alreadyExists) {
      this.toasts = this.toasts.map((t) => {
        if (t.id === id) {
          this.publish({ ...t, ...data, id, title: message } as ToastT);
          return { ...t, ...data, id, dismissible, title: message } as ToastT;
        }
        return t;
      });
    } else {
      this.addToast({ title: message, ...rest, dismissible, id } as ToastT);
    }

    return id;
  };

  dismiss = (id?: string) => {
    if (id !== undefined) {
      this.dismissedToasts.add(id);
      requestAnimationFrame(() => {
        for (const subscriber of this.subscribers) {
          subscriber({ id, dismiss: true });
        }
      });
    } else {
      requestAnimationFrame(() => {
        for (const toast of this.toasts) {
          for (const subscriber of this.subscribers) {
            subscriber({ id: toast.id, dismiss: true });
          }
        }
      });
    }

    return id;
  };

  message = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, message });

  error = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, message, type: "error" });

  success = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, type: "success", message });

  info = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, type: "info", message });

  warning = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, type: "warning", message });

  loading = (message: JSXElement, data?: ExternalToast) =>
    this.create({ ...data, type: "loading", message });

  promise = <ToastData>(
    promise: PromiseT<ToastData>,
    data?: PromiseData<ToastData>,
  ) => {
    if (!data) return;

    let id: string | undefined;
    if (data.loading !== undefined) {
      id = this.create({
        ...data,
        promise,
        type: "loading",
        message: data.loading,
        description:
          typeof data.description !== "function" ? data.description : undefined,
      });
    }

    const p = Promise.resolve(
      promise instanceof Function ? promise() : promise,
    );

    let shouldDismiss = id !== undefined;
    let result: ["resolve", ToastData] | ["reject", unknown];

    const originalPromise = p
      .then(async (response) => {
        result = ["resolve", response];
        const isElement = isValidElement(response as any);
        if (isElement) {
          shouldDismiss = false;
          this.create({ id, type: "default", message: response as any });
        } else if (isHttpResponse(response) && !(response as Response).ok) {
          shouldDismiss = false;
          const errMsg = `HTTP error! status: ${(response as Response).status}`;
          const promiseData =
            typeof data.error === "function"
              ? await data.error(errMsg)
              : data.error;
          const description =
            typeof data.description === "function"
              ? await data.description(errMsg)
              : data.description;
          const isExtended =
            typeof promiseData === "object" &&
            !isValidElement(promiseData as any);
          const toastSettings: PromiseIExtendedResult = isExtended
            ? (promiseData as PromiseIExtendedResult)
            : { message: promiseData as any };
          this.create({ id, type: "error", description, ...toastSettings });
        } else if (response instanceof Error) {
          shouldDismiss = false;
          const promiseData =
            typeof data.error === "function"
              ? await data.error(response)
              : data.error;
          const description =
            typeof data.description === "function"
              ? await data.description(response)
              : data.description;
          const isExtended =
            typeof promiseData === "object" &&
            !isValidElement(promiseData as any);
          const toastSettings: PromiseIExtendedResult = isExtended
            ? (promiseData as PromiseIExtendedResult)
            : { message: promiseData as any };
          this.create({ id, type: "error", description, ...toastSettings });
        } else if (data.success !== undefined) {
          shouldDismiss = false;
          const promiseData =
            typeof data.success === "function"
              ? await data.success(response)
              : data.success;
          const description =
            typeof data.description === "function"
              ? await data.description(response)
              : data.description;
          const isExtended =
            typeof promiseData === "object" &&
            !isValidElement(promiseData as any);
          const toastSettings: PromiseIExtendedResult = isExtended
            ? (promiseData as PromiseIExtendedResult)
            : { message: promiseData as any };
          this.create({ id, type: "success", description, ...toastSettings });
        }
      })
      .catch(async (error) => {
        result = ["reject", error];
        if (data.error !== undefined) {
          shouldDismiss = false;
          const promiseData =
            typeof data.error === "function"
              ? await data.error(error)
              : data.error;
          const description =
            typeof data.description === "function"
              ? await data.description(error)
              : data.description;
          const isExtended =
            typeof promiseData === "object" &&
            !isValidElement(promiseData as any);
          const toastSettings: PromiseIExtendedResult = isExtended
            ? (promiseData as PromiseIExtendedResult)
            : { message: promiseData as any };
          this.create({ id, type: "error", description, ...toastSettings });
        }
      })
      .finally(() => {
        if (shouldDismiss) {
          this.dismiss(id);
          id = undefined;
        }
        data.finally?.();
      });

    const unwrap = () =>
      new Promise<ToastData>((resolve, reject) =>
        originalPromise
          .then(() =>
            result[0] === "reject"
              ? reject(result[1])
              : resolve(result[1] as ToastData),
          )
          .catch(reject),
      );

    if (typeof id !== "string" && typeof id !== "number") return { unwrap };
    return Object.assign(id, { unwrap });
  };

  custom = (jsx: (id: number | string) => any, data?: ExternalToast) => {
    const id = data?.id || createUniqueId();
    this.create({ jsx: jsx(id), ...data, id });
    return id;
  };

  getActiveToasts = () =>
    this.toasts.filter((t) => !this.dismissedToasts.has(t.id));
}

// ─── Singleton & public API ────────────────────────────────────────────────────
export const ToastState = new Observer();

const toastFunction = (message: JSXElement, data?: ExternalToast) => {
  const id = createUniqueId();
  ToastState.addToast({ title: message, ...data, id } as ToastT);
  return id;
};

const isHttpResponse = (data: any): data is Response =>
  data &&
  typeof data === "object" &&
  "ok" in data &&
  typeof data.ok === "boolean" &&
  "status" in data &&
  typeof data.status === "number";

export const getHistory = () => ToastState.toasts;
export const getToasts = () => ToastState.getActiveToasts();

export const toast = Object.assign(
  toastFunction,
  {
    success: ToastState.success,
    info: ToastState.info,
    warning: ToastState.warning,
    error: ToastState.error,
    custom: ToastState.custom,
    message: ToastState.message,
    promise: ToastState.promise,
    dismiss: ToastState.dismiss,
    loading: ToastState.loading,
  },
  { getHistory, getToasts },
);

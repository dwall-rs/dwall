import {
  type Accessor,
  type Setter,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import type { Position, ToastT } from "../Toast/Toast.types";

export interface UseToasterOptions {
  toasts: Accessor<ToastT[]>;

  toasterId: Accessor<string | undefined>;
  position: Accessor<Position>;
  visibleToasts: Accessor<number>;
  expand: Accessor<boolean | undefined>;
  hotkey: Accessor<string[]>;
}

export interface ToasterState {
  expanded: Accessor<boolean>;
  setExpanded: Setter<boolean>;

  possiblePositions: Accessor<Position[]>;

  visibleToastsForPosition: (position: Position) => ToastT[];
}

export function useToaster(options: UseToasterOptions): ToasterState {
  const [expanded, setExpanded] = createSignal(false);

  const filteredToasts = createMemo(() => {
    const id = options.toasterId();
    return id
      ? options.toasts().filter((toast) => toast.toasterId === id)
      : options.toasts().filter((toast) => !toast.toasterId);
  });

  const possiblePositions = createMemo(() =>
    Array.from(
      new Set(
        [options.position()].concat(
          filteredToasts()
            .filter((toast) => toast.position)
            .map((toast) => toast.position as Position),
        ),
      ),
    ),
  );

  const toastsForPosition = (position: Position) =>
    filteredToasts().filter((toast) => {
      if (toast.position) return toast.position === position;
      return position === options.position();
    });

  const visibleToastsForPosition = (position: Position) => {
    const list = toastsForPosition(position);
    if (options.expand()) return list;
    return list.slice(0, options.visibleToasts());
  };

  onMount(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const hotkeyPressed =
        options.hotkey().length > 0 &&
        options
          .hotkey()
          .every((key) => (event as any)[key] || event.code === key);
      if (hotkeyPressed) setExpanded(true);
      if (event.code === "Escape") setExpanded(false);
    };

    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return { expanded, setExpanded, possiblePositions, visibleToastsForPosition };
}

import { type Accessor, createEffect, createSignal, onCleanup } from "solid-js";

export const EXIT_FALLBACK_MS = 300;

export interface UsePresenceOptions {
  visible: Accessor<boolean>;

  element: Accessor<HTMLElement | undefined>;

  consumeSuppressExitAnimation: () => boolean;

  fallbackMs?: number;
}

export function usePresence(options: UsePresenceOptions): Accessor<boolean> {
  const fallbackMs = options.fallbackMs ?? EXIT_FALLBACK_MS;
  const [mounted, setMounted] = createSignal(options.visible());

  createEffect<boolean>((wasVisible) => {
    const visible = options.visible();
    const element = options.element();

    if (visible) {
      setMounted(true);
    } else if (wasVisible && element) {
      if (options.consumeSuppressExitAnimation()) {
        setMounted(false);
      } else {
        const handleAnimationEnd = (event: AnimationEvent) => {
          if (event.target !== element) return;
          setMounted(false);
        };
        element.addEventListener("animationend", handleAnimationEnd);
        onCleanup(() =>
          element.removeEventListener("animationend", handleAnimationEnd),
        );

        const fallback = window.setTimeout(() => setMounted(false), fallbackMs);
        onCleanup(() => window.clearTimeout(fallback));
      }
    } else {
      setMounted(false);
    }

    return visible;
  }, options.visible());

  return mounted;
}

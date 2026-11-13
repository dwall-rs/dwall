import { For, Show, mergeProps, type Component } from "solid-js";
import { Portal } from "solid-js/web";
import { removeToast, useSonner } from "../state/toast";
import type { ToasterProps } from "./Toaster.types";
import { getDocumentDirection } from "./Toaster.utils";
import { ToastViewport } from "./ToastViewport";
import { useToaster } from "./useToaster";

export const Toaster: Component<ToasterProps> = (props) => {
  const merged = mergeProps(
    {
      position: "bottom-right",
      gap: 14,
      visibleToasts: 3,
      dir: getDocumentDirection(),
      containerAriaLabel: "Notifications",
      hotkey: ["altKey", "KeyT"] as string[],
    } as const,
    props,
  );

  const { toasts } = useSonner();

  const { expanded, setExpanded, possiblePositions, visibleToastsForPosition } =
    useToaster({
      toasts: () => toasts,
      toasterId: () => merged.id,
      position: () => merged.position,
      visibleToasts: () => merged.visibleToasts,
      expand: () => merged.expand,
      hotkey: () => merged.hotkey,
    });

  return (
    <Portal>
      <div
        data-slot="toaster"
        dir={merged.dir === "auto" ? getDocumentDirection() : merged.dir}
      >
        <For each={possiblePositions()}>
          {(position) => (
            <Show when={visibleToastsForPosition(position).length > 0}>
              <ToastViewport
                position={position}
                toasts={visibleToastsForPosition(position)}
                expanded={expanded()}
                gap={merged.gap}
                ariaLabel={merged.customAriaLabel ?? merged.containerAriaLabel}
                offset={merged.offset}
                mobileOffset={merged.mobileOffset}
                style={merged.style}
                closeButton={merged.closeButton}
                duration={merged.duration}
                icons={merged.icons}
                variant={merged.variant}
                richColors={merged.richColors}
                toastOptions={merged.toastOptions}
                onExpandChange={setExpanded}
                onRemove={removeToast}
              />
            </Show>
          )}
        </For>
      </div>
    </Portal>
  );
};

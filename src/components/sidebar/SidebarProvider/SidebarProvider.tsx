import {
  createEffect,
  createMemo,
  createSignal,
  mergeProps,
  onCleanup,
  splitProps,
} from "solid-js";
import type { SidebarProviderProps } from "./SidebarProvider.types";
import {
  SIDEBAR_COOKIE_MAX_AGE,
  SIDEBAR_COOKIE_NAME,
  SIDEBAR_KEYBOARD_SHORTCUT,
} from "./SidebarProvider.consts";
import { SidebarContext } from "./SidebarProvider.context";
import { clsx } from "~/utils";

export const SidebarProvider = (props: SidebarProviderProps) => {
  const merged = mergeProps({ defaultOpen: true } as const, props);
  const [local, others] = splitProps(merged, [
    "open",
    "onOpenChange",
    "defaultOpen",
    "onOpenChange",
    "class",
    "style",
    "children",
  ]);

  // FIXME: Remove hardcoded value and use a responsive breakpoint instead.
  const isMobile = false;

  const [openMobile, setOpenMobile] = createSignal(false);

  // This is the internal state of the sidebar.
  // We use openProp and setOpenProp for control from outside the component.
  const [_open, _setOpen] = createSignal(local.defaultOpen);
  const open = createMemo(() => local.open ?? _open());

  const setOpen = (value: boolean | ((value: boolean) => boolean)) => {
    const openState = typeof value === "function" ? value(open()) : value;
    if (local.onOpenChange) {
      local.onOpenChange(openState);
    } else {
      _setOpen(openState);
    }

    // biome-ignore lint/suspicious/noDocumentCookie: This sets the cookie to keep the sidebar state.
    document.cookie = `${SIDEBAR_COOKIE_NAME}=${openState}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
  };

  // Helper to toggle the sidebar.
  const toggleSidebar = () =>
    isMobile ? setOpenMobile((open) => !open) : setOpen((open) => !open);

  createEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === SIDEBAR_KEYBOARD_SHORTCUT &&
        (event.metaKey || event.ctrlKey)
      ) {
        event.preventDefault();
        toggleSidebar();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  // We add a state so that we can do data-state="expanded" or "collapsed".
  // This makes it easier to style the sidebar with Tailwind classes.
  const state = createMemo(() => (open() ? "expanded" : "collapsed"));

  return (
    <SidebarContext.Provider
      value={{
        state,
        open,
        setOpen,
        isMobile,
        openMobile,
        setOpenMobile,
        toggleSidebar,
      }}
    >
      <div
        data-slot="sidebar-wrapper"
        // style={{
        //   "--sidebar-width": SIDEBAR_WIDTH,
        //   "--sidebar-width-icon": SIDEBAR_WIDTH_ICON,
        //   ...local.style,
        // }}
        class={clsx(
          "group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-neutral-50 dark:has-data-[variant=inset]:bg-neutral-900",
          local.class,
        )}
        {...others}
      >
        {local.children}
      </div>
    </SidebarContext.Provider>
  );
};

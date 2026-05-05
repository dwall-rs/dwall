import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-solid";
import { clsx } from "~/utils";

const appWindow = getCurrentWindow();

const Titlebar = () => (
  <div
    class={clsx(
      "h-7 fixed z-50 top-0 left-0 right-0 grid grid-cols-[auto_max-content]",
      "hover:bg-neutral-200/30 dark:hover:bg-neutral-700/30 hover:shadow-xs",
      "active:bg-neutral-300/30 dark:active:bg-neutral-600/30 active:shadow-sm",
      "focus:bg-neutral-300/30 dark:focus:bg-neutral-600/30 focus:shadow-sm",
    )}
  >
    <div data-tauri-drag-region />

    <div>
      <button
        type="button"
        onClick={() => appWindow.minimize()}
        class="inline-flex items-center justify-center rounded-none border-none w-10 h-7 text-sm hover:bg-neutral-50 active:bg-neutral-200 [&_svg]:pointer-events-none [&_svg]:shrink-0"
      >
        <Minus size="16" />
      </button>
      <button
        type="button"
        onClick={() => appWindow.close()}
        class="inline-flex items-center justify-center rounded-none border-none w-10 h-7 hover:bg-red-600 active:bg-red-400 dark:hover:bg-red-400 dark:active:bg-red-600 hover:text-white active:text-white [&_svg]:pointer-events-none [&_svg]:shrink-0"
      >
        <X size="16" />
      </button>
    </div>
  </div>
);

export default Titlebar;

import { onCleanup, onMount, type ChildrenReturn } from "solid-js";
import { useDialogContext } from "../Dialog";

export const useDialogTrigger = (resolved: ChildrenReturn) => {
  const { setShow, setOpen } = useDialogContext();

  const handleClick = (ev: PointerEvent) => {
    console.log("点击", ev.type);
    setOpen(true);
    setShow(true);
  };

  onMount(() => {
    const trigger = resolved.toArray();
    if (trigger.length !== 1) {
      console.warn("[Dialog] DialogTrigger should have exactly one child");
    }
    const el = trigger[0] as HTMLElement | undefined;
    if (!el) return;
    if (!(el instanceof HTMLElement)) {
      console.error("[Dialog] DialogTrigger child should be an HTMLElement");
      return;
    }
    el.addEventListener("click", handleClick);
    onCleanup(() => {
      if (!el) return;
      el.removeEventListener("click", handleClick);
    });
  });
};

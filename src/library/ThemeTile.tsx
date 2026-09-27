// Responsibility: A single theme tile; addition — at the smallest tier in fixed mode the drawer collapses automatically after selection so the stage feedback stays visible.
import { setLibOpen } from "@/store/ui.store";
import { modeStore } from "@/store/mode.store";
import { fixedStore, scopeKey, setMonitorTheme } from "@/store/fixed.store";
import { randomStore, toggle } from "@/store/random.store";
import { useMediaQuery } from "@/hooks/useMediaQuery";
import { MQ_XL } from "@/lib/layout";
import { themeById } from "@/store/catalog.selectors";
import { clsx } from "@/utils";
import { Check } from "lucide-solid";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

interface Props {
  id: string;
  index: number;
}

export function ThemeTile({ id, index }: Props) {
  const isXl = useMediaQuery(MQ_XL);

  const t = themeById(id);
  const on = () =>
    modeStore.mode === "fixed"
      ? fixedStore.monitorThemes[scopeKey()] === id
      : randomStore.selected.includes(id);

  const onClick = () => {
    if (modeStore.mode === "fixed") {
      setMonitorTheme(scopeKey(), id);
      // Smallest tier: collapse the drawer right after assigning, so the user immediately sees the sun-path/preview update; random mode stays open for consecutive checking
      if (!isXl()) setLibOpen(false);
    } else {
      toggle(id);
    }
  };

  return (
    <div
      onClick={onClick}
      style={{ "animation-delay": `${index * 40}ms` }}
      class={clsx(
        "group relative aspect-16/10 cursor-pointer overflow-hidden rounded-lg border transition-all duration-200 animate-rise hover:-translate-y-1 hover:scale-[1.02] hover:border-border-2 hover:shadow-[0_12px_26px_rgba(0,0,0,.5)]",
        on() && modeStore.mode === "fixed" && "border-primary",
        on() && modeStore.mode === "random" && "border-warning",
      )}
    >
      <ThemeThumbnail
        themeId={id}
        class="transition-transform duration-500 group-hover:scale-105"
      />
      <div class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/72 to-transparent px-2 pb-1.5 pt-3.5 font-display text-[10.5px] font-semibold text-white opacity-0 transition-opacity group-hover:opacity-100">
        {t.name}
      </div>
      <span
        class={clsx(
          "absolute right-1.5 top-1.5 grid size-5 place-items-center rounded-md transition-all duration-200",
          on() ? "scale-100 opacity-100" : "scale-50 opacity-0",
          modeStore.mode === "fixed"
            ? "bg-primary text-primary-foreground"
            : "bg-warning text-warning-foreground",
        )}
      >
        <Check class="size-3" strokeWidth={3} />
      </span>
    </div>
  );
}

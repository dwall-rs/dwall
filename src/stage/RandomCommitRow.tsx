// Responsibility: random-mode commit — pool size, unsaved state, and the save that
// writes it to the engine. Only rendered while there is something to commit.
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { randomStore, isDirty, save, discard } from "@/store/random.store";
import { availableThemes } from "@/store/catalog.selectors";
import { createMemo, Show } from "solid-js";
import { LoaderCircle, Save } from "lucide-solid";

export function RandomCommitRow() {
  const dirty = createMemo(() => isDirty(randomStore));
  const canSave = createMemo(
    () => !randomStore.saving && randomStore.selected.length > 0,
  );

  return (
    <Show when={dirty()}>
      <section class="rounded-[16px] border border-warning/25 bg-warning-dim p-3.5">
        <div class="flex items-start gap-2">
          <span class="mt-1 size-1.5 shrink-0 rounded-full bg-warning animate-bdot" />
          <p class="text-[11px] leading-relaxed text-warning">
            {t("scope.randomDirty", {
              n: randomStore.selected.length,
              excluded: Math.max(
                0,
                availableThemes().length - randomStore.selected.length,
              ),
            })}
          </p>
        </div>
        <div class="mt-3 flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={discard}
            disabled={randomStore.saving}
          >
            {t("commit.discard")}
          </Button>
          <Button size="sm" disabled={!canSave()} onClick={save} class="flex-1">
            <Show
              when={randomStore.saving}
              fallback={<Save class="size-3.5" />}
            >
              <LoaderCircle class="size-3.5 animate-spin" />
            </Show>
            {randomStore.saving ? t("commit.saving") : t("themes.savePool")}
          </Button>
        </div>
      </section>
    </Show>
  );
}

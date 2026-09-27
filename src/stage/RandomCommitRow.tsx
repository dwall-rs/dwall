// Responsibility: Commit row for random mode — "save config / discard" is random mode's "apply": it writes the config + restarts the engine.
//       The row keeps a fixed height but only shows the buttons while there are unsaved changes, so the collage above never resizes.
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { randomStore, isDirty, save, discard } from "@/store/random.store";
import { clsx } from "@/utils";
import { LoaderCircle, Save } from "lucide-solid";
import { createMemo, Show } from "solid-js";

export function RandomCommitRow() {
  const dirty = createMemo(() => isDirty(randomStore));
  const canSave = createMemo(
    () => !randomStore.saving && randomStore.selected.length > 0,
  );

  return (
    <div class="flex min-h-12 shrink-0 items-center justify-center gap-3 pb-4 pt-1">
      <Show when={dirty()}>
        <Button variant="ghost" size="sm" onClick={discard}>
          {t("commit.discard")}
        </Button>
        <Button
          size="sm"
          disabled={!canSave()}
          onClick={save}
          class={clsx(
            canSave() && "animate-breathe",
            "px-7 py-2.5 text-[13.5px]",
          )}
        >
          {randomStore.saving ? (
            <>
              <LoaderCircle class="size-3.5 animate-spin" />
              {t("commit.saving")}
            </>
          ) : (
            <>
              <Save class="size-3.5" />
              {t("commit.save")}
            </>
          )}
        </Button>
      </Show>
    </div>
  );
}

// Responsibility: Empty state for the candidate pool — says why it is empty, because
//       "you excluded everything" is wrong when nothing is installed or the catalog
//       could not be read.
import { t } from "@/i18n";
import { load as loadCatalog } from "@/store/catalog.store";
import { Button } from "@/components/ui/button";
import { ImageOff } from "lucide-solid";

export type PoolEmptyReason = "excluded" | "none" | "error";

interface Props {
  reason: PoolEmptyReason;
}

export function PoolEmpty(props: Props) {
  const title = () =>
    props.reason === "error"
      ? t("library.loadFailed")
      : props.reason === "none"
        ? t("stage.noThemesTitle")
        : t("stage.emptyTitle");

  const description = () =>
    props.reason === "error"
      ? t("stage.catalogErrorDesc")
      : props.reason === "none"
        ? t("stage.noThemesDesc")
        : t("stage.emptyDesc");

  return (
    <div class="relative flex min-h-0 flex-1 flex-col items-center justify-center gap-2.5 rounded-2xl border border-dashed border-border-2 text-muted-foreground">
      <ImageOff class="size-10 opacity-40" />
      <div class="font-display text-[15px] font-bold text-foreground/80">
        {title()}
      </div>
      <div class="max-w-75 text-center text-[12px] leading-relaxed">
        {description()}
      </div>
      {/* Only the failed read has an action: the other two are answered by the library. */}
      <Button
        variant="outline"
        size="sm"
        onClick={() => void loadCatalog()}
        class={props.reason === "error" ? "mt-1" : "hidden"}
      >
        {t("library.reload")}
      </Button>
    </div>
  );
}

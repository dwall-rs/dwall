// Responsibility: the one error surface the stages use when a backend read fails, so
// "could not load" is not rendered as "not installed" or "no theme drawn yet".
import { RefreshCw } from "lucide-solid";

import { Button } from "@/components/ui/button";
import { t } from "@/i18n";

interface Props {
  /** The backend message, shown selectable for a bug report. */
  message: string;
  onRetry: () => void;
}

export function StageError(props: Props) {
  return (
    <div class="flex aspect-video w-full flex-col items-center justify-center gap-3 rounded-[18px] border border-destructive/30 bg-destructive-dim px-6 text-center">
      <p class="font-display text-[15px] font-semibold">
        {t("common.operationFailed")}
      </p>
      <p
        data-selectable
        class="max-w-[58ch] font-mono text-[11px] leading-relaxed text-destructive"
      >
        {props.message}
      </p>
      <Button variant="outline" size="sm" onClick={props.onRetry}>
        <RefreshCw class="size-3.5" />
        {t("library.reload")}
      </Button>
    </div>
  );
}

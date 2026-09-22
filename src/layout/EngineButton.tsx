// 职责：引擎进程按钮——运行中=destructive「终止」（二次确认脉动），停止=primary「启动」。
import { Button } from "@/components/ui/button";
import { t } from "@/i18n";
import { engineStore, toggle } from "@/store/engine.store";
import { Play, Square } from "lucide-solid";

export function EngineButton() {
  if (!engineStore.running)
    return (
      <Button
        variant="outline"
        size="sm"
        onClick={toggle}
        class="border-primary/40 text-primary hover:bg-primary/10"
      >
        <Play class="size-3.5 fill-current" />
        {t("app.engine.start")}
      </Button>
    );

  return (
    <Button variant="outline" size="sm" onClick={toggle}>
      {!engineStore.pending && <Square class="size-3 fill-current" />}
      {engineStore.pending ? t("app.engine.confirmStop") : t("app.engine.stop")}
    </Button>
  );
}

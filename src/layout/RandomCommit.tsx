// 职责：随机模式提交状态——顶栏显示「所有显示器 + 已应用今日主题 / 未应用」。
import { t } from "@/i18n";
import { themeById } from "@/domain/themes";
import { getRandomSelection } from "@/ipc";
import { engineStore } from "@/store/engine.store";
import { clsx } from "@/utils";
import { createMemo, createResource } from "solid-js";

export function RandomCommit() {
  const [selection] = createResource(getRandomSelection);

  const appliedTheme = createMemo(() => {
    const selected = selection();
    return engineStore.running && selected
      ? themeById(selected.themeId).name
      : null;
  });

  return (
    <div class="flex items-center gap-2.5 text-[12.5px] text-muted-foreground">
      <span class="font-display font-bold text-foreground">
        {t("scope.allMonitors")}
      </span>
      <span
        class={clsx(
          "font-mono text-[12px]",
          appliedTheme() ? "text-success" : "text-muted-foreground",
        )}
      >
        {appliedTheme()
          ? t("commit.applied", { theme: appliedTheme()! })
          : t("commit.notApplied")}
      </span>
    </div>
  );
}

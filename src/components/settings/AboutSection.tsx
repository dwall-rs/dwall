/* ===== src/components/settings/AboutSection.tsx ===== */
// 职责：关于分组——不等分（一个大版本块 + 一列窄链接行），破「三等卡」默认相。
import { ChevronRight, Code, FileText } from "lucide-solid";
import { createResource, type JSXElement } from "solid-js";
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from "@/components/ui/item";
import { getAppInfo, openLogDir, openUrl } from "@/ipc";
import { t } from "@/i18n";
import Logo from "@/assets/dwall.svg";
import { SettingsGroup } from "./SettingsGroup";
import { UpdateCheck } from "./UpdateCheck";

const SOURCE_URL = "https://github.com/dwall-rs/dwall";

export function AboutSection() {
  const [info] = createResource(getAppInfo);

  return (
    <SettingsGroup
      icon={<FileText class="size-3.25" />}
      title={t("settings.about.title")}
      delay={200}
    >
      <div class="grid grid-cols-[1.5fr_1fr] gap-3">
        <div class="relative row-span-2 flex flex-col gap-2.5 overflow-hidden rounded-[15px] border border-border bg-gradient-to-br from-primary/10 to-card p-5 shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
          <img src={Logo} alt={t("settings.about.logoAlt")} class="size-8.5" />
          <div class="font-display text-[34px] font-extrabold leading-none tracking-tight">
            {info()?.version ?? "…"}
          </div>
          <div class="font-mono text-[11px] text-muted-foreground">
            {t("settings.about.tagline")}
          </div>
          <UpdateCheck />
        </div>
        <LinkRow
          icon={<Code class="size-3.5" />}
          title={t("settings.about.sourceCode")}
          sub={t("settings.about.sourceSub")}
          onClick={() => void openUrl(SOURCE_URL)}
        />
        <LinkRow
          icon={<FileText class="size-3.5" />}
          title={t("settings.about.logDir")}
          sub={t("settings.about.logDirSub")}
          onClick={() => void openLogDir()}
        />
      </div>
    </SettingsGroup>
  );
}
function LinkRow(props: {
  icon: JSXElement;
  title: string;
  sub: string;
  onClick: () => void;
}) {
  return (
    <Item
      component="button"
      variant="outline"
      onClick={props.onClick}
      class="bg-card text-left shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)] transition-all hover:translate-x-0.5 hover:border-border-2 hover:bg-secondary"
    >
      <ItemMedia class="size-7 rounded-lg bg-accent text-primary">
        {props.icon}
      </ItemMedia>
      <ItemContent>
        <ItemTitle class="text-[13px] font-bold!">{props.title}</ItemTitle>
        <ItemDescription class="text-[11px]">{props.sub}</ItemDescription>
      </ItemContent>
      <ItemActions class="ml-auto">
        <ChevronRight class="size-3.5 text-muted-foreground" />
      </ItemActions>
    </Item>
  );
}

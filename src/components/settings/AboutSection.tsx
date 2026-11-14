// Responsibility: About group — version and update on one side, reference links on
// the other. Kept deliberately quiet: it is the last thing on the settings page.
import { ChevronRight, Code, FileText, ScrollText } from "lucide-solid";
import { createResource, type JSXElement } from "solid-js";
import { getAppInfo, openLogDir, openUrl } from "@/ipc";
import { resourceValue } from "@/lib/resource";
import { t } from "@/i18n";
import Logo from "@/assets/dwall.svg";
import { SettingsGroup } from "./SettingsGroup";
import { UpdateCheck } from "./UpdateCheck";

const SOURCE_URL = "https://github.com/dwall-rs/dwall";
const CHANGELOG_URL = `${SOURCE_URL}/releases`;

export function AboutSection() {
  const [info] = createResource(getAppInfo);

  return (
    <SettingsGroup
      id="settings-about"
      icon={<FileText class="size-3.5" />}
      title={t("settings.about.title")}
      bare
    >
      <div class="grid gap-3 min-[900px]:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)]">
        <div class="relative flex flex-col gap-2.5 overflow-hidden rounded-[16px] border border-border bg-linear-to-br from-accent/70 to-card p-5">
          <div class="pointer-events-none absolute -right-10 -top-12 size-40 rounded-full bg-solar/10 blur-2xl" />
          <img
            src={Logo}
            alt={t("settings.about.logoAlt")}
            class="size-8 drag-none"
          />
          <div class="font-display text-[34px] font-semibold leading-none tracking-[-0.035em]">
            {resourceValue(info)?.version ?? "…"}
          </div>
          <div class="font-mono text-[10.5px] text-muted-foreground">
            {t("settings.about.tagline")}
          </div>
          <UpdateCheck />
        </div>

        <div class="flex flex-col overflow-hidden rounded-[16px] border border-border bg-card">
          <LinkRow
            icon={<ScrollText class="size-4" />}
            title={t("settings.about.changelog")}
            sub={t("settings.about.changelogSub")}
            onClick={() => void openUrl(CHANGELOG_URL)}
          />
          <LinkRow
            icon={<Code class="size-4" />}
            title={t("settings.about.sourceCode")}
            sub={t("settings.about.sourceSub")}
            onClick={() => void openUrl(SOURCE_URL)}
          />
          <LinkRow
            icon={<FileText class="size-4" />}
            title={t("settings.about.logDir")}
            sub={t("settings.about.logDirSub")}
            onClick={() => void openLogDir()}
          />
        </div>
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
    <button
      type="button"
      onClick={props.onClick}
      class="group flex cursor-pointer items-center gap-3 border-b border-border px-3.5 py-3 text-left transition-colors last:border-b-0 hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
    >
      <span class="grid size-8 shrink-0 place-items-center rounded-[9px] bg-accent text-accent-foreground">
        {props.icon}
      </span>
      <span class="min-w-0 flex-1">
        <span class="block text-[12.5px] font-semibold">{props.title}</span>
        <span class="mt-0.5 block text-[10.5px] text-muted-foreground">
          {props.sub}
        </span>
      </span>
      <ChevronRight class="size-3.5 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
    </button>
  );
}

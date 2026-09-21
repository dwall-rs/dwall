/* ===== src/components/settings/AboutSection.tsx ===== */
// 职责：关于分组——不等分（一个大版本块 + 一列窄链接行），破「三等卡」默认相。
import { ChevronRight, Code, FileText } from "lucide-solid";
import { createResource, type JSXElement } from "solid-js";
import { getAppInfo, openLogDir, openUrl } from "@/ipc";
import Logo from "@/assets/dwall.svg";
import { SettingsGroup } from "./SettingsGroup";

const SOURCE_URL = "https://github.com/dwall-rs/dwall";

export function AboutSection() {
  const [info] = createResource(getAppInfo);

  return (
    <SettingsGroup
      icon={<FileText class="size-3.25" />}
      title="关于"
      delay={200}
    >
      <div class="grid grid-cols-[1.5fr_1fr] gap-3">
        <div class="relative row-span-2 flex flex-col gap-2.5 overflow-hidden rounded-[15px] border border-border bg-[linear-gradient(135deg,hsl(var(--primary)/.10),hsl(var(--card)))] p-5 shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
          <img src={Logo} alt="Dwall Logo" class="size-8.5" />
          <div class="font-display text-[34px] font-extrabold leading-none tracking-tight">
            {info()?.version ?? "…"}
          </div>
          <div class="font-mono text-[11px] text-muted-foreground">
            Dwall · 太阳位置驱动
          </div>
        </div>
        <LinkRow
          icon={<Code class="size-3.5" />}
          title="源代码"
          sub="github.com"
          onClick={() => void openUrl(SOURCE_URL)}
        />
        <LinkRow
          icon={<FileText class="size-3.5" />}
          title="日志目录"
          sub="打开文件夹"
          onClick={() => void openLogDir()}
        />
      </div>
    </SettingsGroup>
  );
}
function LinkRow({
  icon,
  title,
  sub,
  onClick,
}: {
  icon: JSXElement;
  title: string;
  sub: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      class="flex items-center gap-3 rounded-xl border border-border bg-card p-[14px_16px] text-left shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)] transition-all hover:translate-x-0.5 hover:border-border-2 hover:bg-secondary"
    >
      <span class="grid size-7 shrink-0 place-items-center rounded-lg bg-accent text-primary">
        {icon}
      </span>
      <span>
        <span class="block font-display text-[13px] font-bold">{title}</span>
        <span class="block font-mono text-[11px] text-muted-foreground">
          {sub}
        </span>
      </span>
      <ChevronRight class="ml-auto size-3.5 text-muted-foreground" />
    </button>
  );
}

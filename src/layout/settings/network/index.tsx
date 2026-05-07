import { createMemo, createSignal, Show } from "solid-js";
import { ChevronRight } from "lucide-solid";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "~/components/collapsible";
import { Switch } from "~/components/switch";
import { t } from "~/i18n";
import SettingsItem from "../SettingsItem";
import Socks5 from "./Socks5";
import { useConfig } from "~/contexts";
import GithubMirror from "./GithubMirror";

const isSocks5 = (network?: Network) =>
  !(network && typeof network === "string");

const Network = () => {
  const { data: config } = useConfig();

  const isSocks5Value = createMemo(() => isSocks5(config()?.network));

  const [useSocks5, setUseSocks5] = createSignal(isSocks5Value());

  return (
    <Collapsible>
      <CollapsibleTrigger variant="ghost" class="w-full">
        {t("settings.label.network")}
        <ChevronRight class="ml-auto group-data-[panel-open=true]/button:rotate-90" />
      </CollapsibleTrigger>

      <CollapsibleContent class="flex flex-col items-start gap-2 p-2.5 pt-2 text-sm">
        <SettingsItem label={t("settings.label.useSocks5")}>
          <Switch
            checked={useSocks5()}
            onCheckedChange={(checked) => setUseSocks5(checked)}
          />
        </SettingsItem>

        <Show
          when={useSocks5()}
          fallback={
            <GithubMirror
              defaultValue={
                isSocks5Value() ? undefined : (config()?.network as string)
              }
            />
          }
        >
          <Socks5
            defaultValue={
              isSocks5Value() ? (config()?.network as Socks5) : undefined
            }
          />
        </Show>
      </CollapsibleContent>
    </Collapsible>
  );
};

export default Network;

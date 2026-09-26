/* ===== src/components/settings/PathsSection.tsx ===== */
// 职责：目录与下载源分组——主题目录（选择/迁移）+ 网络（镜像模板 / SOCKS5）。
import { Button } from "@/components/ui/button";
import {
  type NetworkType,
  networkType,
  patch,
  patchSocks5,
  save,
  setNetworkType,
  settingsStore,
} from "@/store/settings.store";
import { isSocks5 } from "@/domain/config";
import { t } from "@/i18n";
import { moveDirectory, pickDirectory } from "@/ipc";
import { logger } from "@/utils";
import { SettingsGroup } from "./SettingsGroup";
import { SettingsRow } from "./SettingsRow";
import { createMemo, Match, Switch } from "solid-js";
import { Folder, FolderOpen, LoaderCircle, Save } from "lucide-solid";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "../ui/input-group";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { Field, FieldGroup, FieldLabel } from "../ui/field";
import { Input } from "../ui/input";

const log = logger.child("paths");

export function PathsSection() {
  const network = createMemo(networkType);
  const socks = createMemo(() => {
    const n = settingsStore.config?.network;
    return isSocks5(n) ? n : { host: "", port: 1080 };
  });

  const chooseDir = async () => {
    const dir = await pickDirectory();
    if (!dir) return;

    const current = settingsStore.config?.themes_directory;
    try {
      if (current && current !== dir) {
        await moveDirectory(current, dir);
      }
      patch({ themes_directory: dir });
      await save();
    } catch (e) {
      log.error("Failed to move themes directory", e);
    }
  };

  return (
    <SettingsGroup
      icon={<Folder class="size-3.25" />}
      title={t("settings.paths.title")}
      delay={140}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          stacked
          label={t("settings.paths.themesDir")}
          desc={t("settings.paths.themesDirDesc")}
          control={
            <div class="flex justify-between gap-2">
              <div class="flex flex-1 items-center gap-2 overflow-hidden rounded-lg border border-border-2 bg-secondary px-3 py-2.25 font-mono text-[12px] text-muted-foreground">
                <Folder class="size-3.5 shrink-0 text-muted-foreground" />
                <span class="truncate">
                  {settingsStore.config?.themes_directory}
                </span>
              </div>
              <div class="flex items-center justify-between gap-4.5">
                <Button
                  variant="outline"
                  size="sm"
                  class="shadow-[inset_0_0_0_1px_var(--border-2)]"
                  disabled={settingsStore.saving}
                  onClick={() => void chooseDir()}
                >
                  <FolderOpen class="size-3.5" />
                  {t("settings.paths.selectDir")}
                </Button>
              </div>
            </div>
          }
        />

        <SettingsRow
          label={t("settings.paths.network")}
          desc={t("settings.paths.networkDesc")}
          control={
            <Tabs
              value={network()}
              onValueChange={(t) => {
                setNetworkType(t as NetworkType);
              }}
            >
              <TabsList>
                <TabsTrigger value="none">
                  {t("settings.paths.none")}
                </TabsTrigger>
                <TabsTrigger value="mirror">
                  {t("settings.paths.mirror")}
                </TabsTrigger>
                <TabsTrigger value="socks5">
                  {t("settings.paths.socks5")}
                </TabsTrigger>
              </TabsList>
            </Tabs>
          }
        />

        <Switch>
          <Match when={network() === "mirror"}>
            <SettingsRow
              stacked
              label={t("settings.paths.mirrorTemplate")}
              desc={
                <>
                  {t("settings.paths.mirrorDesc")}{" "}
                  <span class="cursor-pointer text-primary underline">
                    {t("settings.paths.viewTemplates")}
                  </span>
                </>
              }
              control={
                <InputGroup>
                  <InputGroupInput
                    value={settingsStore.config?.network as string}
                    onChange={(v) => patch({ network: v })}
                    placeholder="https://mirror.example.com/{repo}"
                    class="font-mono text-[12px]"
                  />
                  <InputGroupAddon align="inline-end">
                    <InputGroupButton
                      disabled={settingsStore.saving}
                      icon={
                        settingsStore.saving ? (
                          <LoaderCircle class="size-3.75 animate-spin" />
                        ) : (
                          <Save class="size-3.75" />
                        )
                      }
                      onClick={() => void save()}
                    />
                  </InputGroupAddon>
                </InputGroup>
              }
            />
          </Match>
          <Match when={network() === "socks5"}>
            <SettingsRow
              stacked
              label="SOCKS5"
              control={
                <div class="flex items-end gap-3">
                  <FieldGroup class="grid flex-1 grid-cols-2 gap-3">
                    <Field>
                      <FieldLabel>{t("settings.paths.address")}</FieldLabel>
                      <Input
                        value={socks().host}
                        onChange={(v) => patchSocks5({ host: v })}
                        placeholder="127.0.0.1"
                        class="font-mono text-[12px]"
                      />
                    </Field>
                    <Field>
                      <FieldLabel>{t("settings.paths.port")}</FieldLabel>
                      <Input
                        type="number"
                        value={socks().port}
                        onChange={(v) => patchSocks5({ port: v })}
                        class="font-mono text-[12px]"
                      />
                    </Field>
                  </FieldGroup>
                  <Button
                    size="sm"
                    disabled={settingsStore.saving}
                    onClick={() => void save()}
                  >
                    <Save class="size-3.5" />
                    {t("settings.paths.save")}
                  </Button>
                </div>
              }
            />
          </Match>
        </Switch>
      </div>
    </SettingsGroup>
  );
}

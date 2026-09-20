/* ===== src/components/settings/PathsSection.tsx ===== */
// 职责：目录与下载源分组——主题目录 + 镜像模板（行内保存）。
import { Button } from "@/components/ui/button";
import {
  type NetworkType,
  networkType,
  patch,
  save,
  setNetworkType,
  settingsStore,
} from "@/store/settings.store";
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

export function PathsSection() {
  const network = createMemo(networkType);

  return (
    <SettingsGroup
      icon={<Folder class="size-3.25" />}
      title="目录与下载源"
      delay={140}
    >
      <div class="overflow-hidden rounded-[15px] border border-border bg-card shadow-[0_1px_2px_rgba(0,0,0,.05),0_8px_24px_rgba(0,0,0,.06)]">
        <SettingsRow
          stacked
          label="主题目录"
          desc="本地主题与缩略图的存放位置。"
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
                  class="shadow-[inset_0_0_0_1px_hsl(var(--border-2))]"
                >
                  <FolderOpen class="size-3.5" />
                  选择目录
                </Button>
              </div>
            </div>
          }
        />

        <SettingsRow
          label="网络设置"
          desc={
            <>
              下载主题或者加载缩略图失败时可能需要配置网络，包括 Github
              镜像模板和 SOCKS5 代理。
            </>
          }
          control={
            <Tabs
              value={network()}
              onValueChange={(t) => {
                setNetworkType(t as NetworkType);
              }}
            >
              <TabsList>
                <TabsTrigger value="none">关闭</TabsTrigger>
                <TabsTrigger value="mirror">镜像</TabsTrigger>
                <TabsTrigger value="socks5">SOCKS5</TabsTrigger>
              </TabsList>
            </Tabs>
          }
        />

        <Switch>
          <Match when={network() === "mirror"}>
            <SettingsRow
              stacked
              label="Github 镜像模板"
              desc={
                <>
                  镜像模板用于加速下载。部分国家或地区因网络限制访问 Github
                  可能失败，需配置镜像模板。点此查看可用模板：
                  <span class="cursor-pointer text-primary underline">
                    查看模板列表 ↗
                  </span>
                </>
              }
              control={
                <InputGroup>
                  <InputGroupInput
                    value={settingsStore.config!.network as string}
                    onChange={(v) => patch({ network: v })}
                    placeholder="https://mirror.example.com/{repo}"
                    class="font-mono text-[12px]"
                  />
                  <InputGroupAddon align="inline-end">
                    <InputGroupButton
                      disabled={!settingsStore.config?.network}
                      icon={
                        settingsStore.saving ? (
                          <LoaderCircle class="size-3.75 animate-spin" />
                        ) : (
                          <Save class="size-3.75" />
                        )
                      }
                      onClick={() => save()}
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
                <FieldGroup class="grid grid-cols-2">
                  <Field>
                    <FieldLabel>地址</FieldLabel>
                    <Input
                      defaultValue={
                        (settingsStore.config!.network as Socks5).host
                      }
                    />
                  </Field>

                  <Field>
                    <FieldLabel>端口</FieldLabel>
                    <Input
                      type="number"
                      defaultValue={
                        (settingsStore.config!.network as Socks5).port
                      }
                    />
                  </Field>
                </FieldGroup>
              }
            />
          </Match>
        </Switch>
      </div>
    </SettingsGroup>
  );
}

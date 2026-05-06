import { createMemo, For, Show } from "solid-js";
import { createStore } from "solid-js/store";

import { CircleFadingArrowUp, Settings } from "lucide-solid";

import { useConfig, useTheme, useUpdate } from "~/contexts";

import ThemeImage from "~/components/Image";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "~/components/sidebar";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/tooltip";
import { Spinner } from "~/components/spinner";
import Updater from "~/components/Update";

import { generateGitHubThumbnailMirrorUrl } from "~/utils/proxy";

import { type ThemeItem, themes } from "~/themes";

import { route, navigate } from "~/router";
import { t } from "~/i18n";
import { clsx } from "~/utils";
import { Skeleton } from "~/components/skeleton";

const [imageHeights, setImageHeights] = createStore<Record<string, number>>({});

export const AppSidebar = () => {
  const { data: config } = useConfig();
  const theme = useTheme();
  const { update: updateIsAvailable } = useUpdate();

  const activeThemeID = createMemo(() => {
    const r = route();
    if (r.path === "settings") return null;
    return r.id;
  });

  return (
    <Sidebar
      collapsible="none"
      class="[--sidebar-width:--spacing(24)] flex sticky items-center bg-neutral-200 dark:bg-transparent pt-3 h-screen"
    >
      <SidebarContent
        class={clsx(
          "overflow-x-hidden",
          !theme.downloadingTheme() ? "scrollbar" : "overflow-y-hidden",
        )}
      >
        <SidebarMenu class="gap-2 mx-1.5">
          <For each={themes}>
            {(item, index) => (
              <Show when={config.state === "ready"} fallback={<Spinner />}>
                <ThemeMenuItem
                  {...item}
                  index={index()}
                  active={activeThemeID() === item.id}
                  applied={theme.appliedThemeID() === item.id}
                  github_mirror_template={config()?.github_mirror_template}
                  disabled={theme.downloadingTheme()}
                />
              </Show>
            )}
          </For>
        </SidebarMenu>
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu class="space-y-1">
          <Show when={updateIsAvailable()}>
            <SidebarMenuItem>
              <Updater>
                <Tooltip>
                  <TooltipTrigger>
                    <SidebarMenuButton>
                      <CircleFadingArrowUp />
                    </SidebarMenuButton>
                  </TooltipTrigger>
                  <TooltipContent side="right">
                    {t("sidebar.tooltip.newVersionAvailable")}
                  </TooltipContent>
                </Tooltip>
              </Updater>
            </SidebarMenuItem>
          </Show>

          <SidebarMenuItem>
            <Tooltip>
              <TooltipTrigger>
                <SidebarMenuButton
                  onClick={() => navigate({ path: "settings" })}
                  disabled={theme.downloadingTheme()}
                >
                  <Settings />
                </SidebarMenuButton>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t("sidebar.tooltip.settings")}
              </TooltipContent>
            </Tooltip>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
};

const ThemeMenuItem = (
  props: ThemeItem & {
    github_mirror_template?: string;
    disabled?: boolean;
    active: boolean;
    applied: boolean;
    index: number;
  },
) => {
  return (
    <SidebarMenuItem>
      <Tooltip>
        <TooltipTrigger>
          <SidebarMenuButton
            class="p-1.5 rounded-sm size-18 transition-transform active:translate-y-px border-l-0 border-l-neutral-600 relative"
            isActive={props.active}
            disabled={props.disabled}
            onClick={() => navigate({ path: "theme", id: props.id })}
          >
            <ThemeImage
              class="rounded-sm"
              src={generateGitHubThumbnailMirrorUrl(
                props.thumbnail[0],
                props.github_mirror_template,
              )}
              width={64}
              themeID={props.id}
              serialNumber={props.index + 1}
              height={imageHeights[props.id] ?? 64}
              onLoad={({ height }) => {
                if (height && height !== imageHeights[props.id]) {
                  setImageHeights(props.id, height);
                }
              }}
              skeleton={<Skeleton class="absolute w-15 h-15" />}
            />

            <Show when={props.applied}>
              <div class="absolute top-1/2 -translate-y-1/2 left-0 h-8 w-1 rounded-full bg-neutral-600 dark:bg-neutral-300" />
            </Show>
          </SidebarMenuButton>
        </TooltipTrigger>
        <TooltipContent side="right">{props.id}</TooltipContent>
      </Tooltip>
    </SidebarMenuItem>
  );
};

import { mergeProps, Show, splitProps } from "solid-js";
import type { SidebarMenuSkeletonProps } from "./SidebarMenuSkeleton.types";
import { clsx } from "~/utils";
import { Skeleton } from "~/components/skeleton";

export const SidebarMenuSkeleton = (props: SidebarMenuSkeletonProps) => {
  const merged = mergeProps({ showIcon: false } as const, props);

  const [local, others] = splitProps(merged, [
    "showIcon",
    "class",
    "classList",
  ]);

  const width = () => `${Math.floor(Math.random() * 40) + 50}%`;

  return (
    <div
      data-slot="sidebar-menu-skeleton"
      data-sidebar="menu-skeleton"
      class={clsx("flex h-8 items-center gap-2 rounded-md px-2", local.class)}
      {...others}
    >
      <Show when={local.showIcon}>
        <Skeleton class="size-4 rounded-md" data-sidebar="menu-skeleton-icon" />
      </Show>

      <Skeleton
        class="h-4 max-w-(--skeleton-width) flex-1"
        data-sidebar="menu-skeleton-text"
        style={{
          "--skeleton-width": width(),
        }}
      />
    </div>
  );
};

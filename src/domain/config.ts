// Frontend-facing config types. The Rust `Config` is the single source of truth; these
// are re-exported from the ts-rs generated bindings (see src/ipc/bindings).
export type {
  Config,
  ImageFormat,
  MonitorSpecificWallpapers,
  Network,
  PositionSource,
  WallpaperMode,
} from "@/ipc/types";

import type { Network, PositionSource } from "@/ipc/types";

export type PositionSourceAutomatic = Extract<
  PositionSource,
  { type: "AUTOMATIC" }
>;
export type PositionSourceManual = Extract<PositionSource, { type: "MANUAL" }>;

export interface Socks5 {
  host: string;
  port: number;
}

export const isSocks5 = (n: Network | undefined): n is Socks5 =>
  !!n && typeof n === "object" && "host" in n;

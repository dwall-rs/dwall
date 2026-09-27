import { invoke } from "@tauri-apps/api/core";

import type { PositionSource, SolarPosition } from "./types";

export const getSolarPosition = async (
  positionSource: PositionSource,
  timestamp: number,
): Promise<SolarPosition> =>
  invoke("get_solar_position", { positionSource, timestamp });

export const currentSolarPosition = async (
  positionSource: PositionSource,
): Promise<SolarPosition> =>
  getSolarPosition(positionSource, Math.floor(Date.now() / 1000));

/** Apparent solar positions sampled over a full day, centred on `timestamp`. */
export const getSolarPath = async (
  positionSource: PositionSource,
  timestamp: number,
): Promise<SolarPosition[]> =>
  invoke("get_solar_path", { positionSource, timestamp });

import { describe, expect, test } from "vitest";

import type { DisplayMonitor } from "@/ipc";
import { sortMonitors } from "./monitors";

const monitor = (
  device_path: string,
  position_index: number | null,
): DisplayMonitor => ({
  device_path,
  friendly_name: device_path,
  position_index,
});

describe("sortMonitors", () => {
  test("follows the position Windows enumerated the displays in", () => {
    const sorted = sortMonitors([
      monitor("\\?DISPLAY#2", 2),
      monitor("\\?DISPLAY#0", 0),
      monitor("\\?DISPLAY#1", 1),
    ]);

    expect(sorted.map((item) => item.device_path)).toEqual([
      "\\?DISPLAY#0",
      "\\?DISPLAY#1",
      "\\?DISPLAY#2",
    ]);
  });

  test("uses the device path as a stable tie-break and puts unknown positions last", () => {
    const sorted = sortMonitors([
      monitor("\\?DISPLAY#b", 0),
      monitor("\\?DISPLAY#a", 0),
      monitor("\\?DISPLAY#unknown", null),
    ]);

    expect(sorted.map((item) => item.device_path)).toEqual([
      "\\?DISPLAY#a",
      "\\?DISPLAY#b",
      "\\?DISPLAY#unknown",
    ]);
  });
});

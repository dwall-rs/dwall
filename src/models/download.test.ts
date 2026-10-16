import { describe, expect, test } from "vitest";

import { downloadPercent, isCancellation } from "./download";

describe("downloadPercent", () => {
  test("rounds progress to whole percent", () => {
    expect(downloadPercent({ downloadedBytes: 50, totalBytes: 200 })).toBe(25);
    expect(downloadPercent({ downloadedBytes: 1, totalBytes: 3 })).toBe(33);
  });

  test("is null when the size is unknown, so the bar stays indeterminate", () => {
    expect(downloadPercent({ downloadedBytes: 0, totalBytes: 0 })).toBeNull();
    expect(downloadPercent({ downloadedBytes: 512, totalBytes: 0 })).toBeNull();
  });

  test("clamps to 100 when a retry re-downloads bytes already counted", () => {
    expect(downloadPercent({ downloadedBytes: 200, totalBytes: 200 })).toBe(
      100,
    );
    expect(downloadPercent({ downloadedBytes: 260, totalBytes: 200 })).toBe(
      100,
    );
  });
});

describe("isCancellation", () => {
  test("recognises the backend's cancellation error", () => {
    expect(isCancellation("Download cancelled")).toBe(true);
    expect(isCancellation(new Error("Download cancelled"))).toBe(true);
  });

  test("treats anything else as a failure", () => {
    expect(isCancellation("HTTP 404")).toBe(false);
    expect(isCancellation(undefined)).toBe(false);
  });
});

import { beforeEach, describe, expect, test, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  followsWindows: undefined as boolean | undefined,
}));

vi.mock("./settings.store", () => ({
  settingsStore: {
    get config() {
      return mocks.followsWindows === undefined
        ? null
        : { title_bar_color_follows_windows_theme: mocks.followsWindows };
    },
  },
}));

const mocks2 = vi.hoisted(() => ({
  border: false,
}));

vi.mock("./ui.store", () => ({
  uiStore: {
    get titlebarBorder() {
      return mocks2.border;
    },
  },
}));

async function freshSelectors() {
  vi.resetModules();

  return await import("./ui.selectors");
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.followsWindows = undefined;
  mocks2.border = false;
});

describe("ui selectors: the title bar separator", () => {
  test("is offered while the title bar is painted like the content", async () => {
    mocks.followsWindows = false;
    const selectors = await freshSelectors();

    expect(selectors.titleBarMatchesContent()).toBe(true);
  });

  test("is not offered once the title bar follows the Windows theme", async () => {
    mocks.followsWindows = true;
    const selectors = await freshSelectors();

    expect(selectors.titleBarMatchesContent()).toBe(false);
  });

  test("is not offered before the configuration has loaded", async () => {
    // Undefined means "not read yet", and the window opens before the settings do.
    const selectors = await freshSelectors();

    expect(selectors.titleBarMatchesContent()).toBe(false);
  });

  test("is drawn when the preference is set and the title bar has no colour of its own", async () => {
    mocks.followsWindows = false;
    mocks2.border = true;
    const selectors = await freshSelectors();

    expect(selectors.showsTitlebarBorder()).toBe(true);
  });

  /**
   * The case a hidden row alone would leave broken: a preference saved while the border
   * made sense must not keep drawing it, invisibly, after the title bar gains a colour of
   * its own — the switch that could turn it off is gone by then.
   */
  test("is not drawn once the title bar follows the Windows theme, even if it was asked for", async () => {
    mocks.followsWindows = true;
    mocks2.border = true;
    const selectors = await freshSelectors();

    expect(selectors.showsTitlebarBorder()).toBe(false);
  });

  test("is not drawn when the preference is off", async () => {
    mocks.followsWindows = false;
    mocks2.border = false;
    const selectors = await freshSelectors();

    expect(selectors.showsTitlebarBorder()).toBe(false);
  });
});

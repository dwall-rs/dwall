import { describe, expect, it } from "vitest";

import { callEventHandler } from "./call-event-handler";

describe("callEventHandler", () => {
  it("calls a plain handler with the event", () => {
    let received: Event | null = null;
    const event = new Event("click");

    callEventHandler((e: Event) => {
      received = e;
    }, event);

    expect(received).toBe(event);
  });

  it("calls a bound handler with its data and the event", () => {
    const calls: Array<[unknown, Event]> = [];
    const event = new Event("click");
    const data = { id: 7 };

    callEventHandler(
      [data, (d: unknown, e: Event) => calls.push([d, e])],
      event,
    );

    expect(calls).toEqual([[data, event]]);
  });

  it("ignores missing handlers and non-function values", () => {
    const event = new Event("click");

    expect(() => callEventHandler(undefined, event)).not.toThrow();
    expect(() => callEventHandler(null, event)).not.toThrow();
    expect(() => callEventHandler("nope", event)).not.toThrow();
    expect(() => callEventHandler([{ id: 1 }, undefined], event)).not.toThrow();
  });
});

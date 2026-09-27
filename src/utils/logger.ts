/**
 * Minimal logging library
 *
 * Two APIs, pick by scenario:
 *
 * 1. logger.debug/info/warn/error — exact line numbers
 *    Binds the native console methods directly, so the browser console points at the
 *    real business-code call line and can be clicked to jump there. In production
 *    (import.meta.env.DEV === false) they become no-ops, eliminated as dead code at
 *    build time: negligible size cost, no runtime overhead.
 *    Suitable for: ordinary business logs, not called frequently.
 *
 * 2. logger.hot(tag) — high-frequency hot-path variant
 *    For loops, high-frequency event callbacks, and other scenarios that emit huge
 *    volumes of logs. Trades exact line numbers (the console shows a spot inside
 *    hot(), but the tag prefix makes it easy to locate) for:
 *      - throttling: a given tag emits at most maxPerWindow messages per windowMs
 *        window; extras are dropped outright, and a "skipped N messages" summary is
 *        printed when the window ends, avoiding console spam and main-thread blocking.
 *      - lazy argument evaluation: arguments may be functions, invoked only when the
 *        message is actually emitted; throttled-out calls pay no string
 *        concatenation / JSON.stringify cost.
 *    Suitable for: loop bodies, high-frequency WebSocket messages, frequently
 *    triggered event callbacks, etc.
 *
 * Usage:
 *   import { logger } from './logger';
 *
 *   // ordinary log, exact line number
 *   logger.info('user logged in', { userId: 123 });
 *
 *   // hot-path log, automatic throttling + lazy evaluation
 *   const logFrame = logger.hot('render-loop', 'debug', { windowMs: 1000, maxPerWindow: 3 });
 *   for (const item of hugeList) {
 *     logFrame(() => `processing ${item.id}: ${JSON.stringify(item)}`); // JSON.stringify only runs when actually emitted
 *   }
 */

export type LogLevel = "debug" | "info" | "warn" | "error";

export interface LoggerOptions {
  /** Minimum output level, defaults to debug (i.e. everything is emitted) */
  level?: LogLevel;
  /** Log prefix used to distinguish modules, e.g. 'auth' / 'api' */
  prefix?: string;
}

export interface HotOptions {
  /** Throttle window in milliseconds, defaults to 1000 */
  windowMs?: number;
  /** Maximum messages actually emitted per window, defaults to 5 */
  maxPerWindow?: number;
}

/** hot() arguments may be plain values or lazy thunks (invoked only when actually emitted) */
type HotArg = unknown | (() => unknown);
type HotLogFn = (...args: HotArg[]) => void;
type LogFn = (...args: unknown[]) => void;

const LEVEL_WEIGHT: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

const LEVEL_STYLE: Record<LogLevel, string> = {
  debug: "color:#888",
  info: "color:#2b8a3e",
  warn: "color:#e8590c",
  error: "color:#c92a2a;font-weight:bold",
};

// Vite replaces this line with a literal boolean at build time
const isDev = import.meta.env.DEV;

const noop: LogFn = () => {};
const noopHot: HotLogFn = () => {};

interface HotState {
  count: number;
  windowStart: number;
  suppressed: number;
}

class Logger {
  /** Exact-line-number variant: decided at construction whether it really binds to console or to a no-op */
  readonly debug: LogFn;
  readonly info: LogFn;
  readonly warn: LogFn;
  readonly error: LogFn;

  private level: LogLevel;
  private prefix: string;
  private hotState = new Map<string, HotState>();

  constructor(options: LoggerOptions = {}) {
    this.level = options.level ?? "debug";
    this.prefix = options.prefix ?? "";

    this.debug = this.build("debug", console.debug);
    this.info = this.build("info", console.info);
    this.warn = isDev
      ? this.build("warn", console.warn)
      : (console.warn.bind(console, ...this.tagArgs("warn")) as LogFn);
    // error keeps its output in production by default (for online debugging / error reporting)
    // To be fully silent in production too, replace the line below with this.build('error', console.error)
    this.error = isDev
      ? this.build("error", console.error)
      : (console.error.bind(console, ...this.tagArgs("error")) as LogFn);
  }

  private tagArgs(level: LogLevel): unknown[] {
    const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
    const tag = this.prefix ? `[${this.prefix}]` : "";
    return [`%c${time} ${tag} ${level.toUpperCase()}`, LEVEL_STYLE[level]];
  }

  private build(level: LogLevel, method: (...args: unknown[]) => void): LogFn {
    if (!isDev) return noop;
    if (LEVEL_WEIGHT[level] < LEVEL_WEIGHT[this.level]) return noop;
    return method.bind(console, ...this.tagArgs(level)) as LogFn;
  }

  /**
   * High-frequency hot-path logging: throttling + lazy evaluation, sacrificing exact line numbers.
   * Each call to hot() returns a function bound to the tag; create it once outside the
   * loop / high-frequency callback and reuse it rather than calling logger.hot(...) every iteration.
   */
  hot(
    tag: string,
    level: LogLevel = "debug",
    options: HotOptions = {},
  ): HotLogFn {
    if (!isDev) return noopHot;
    if (LEVEL_WEIGHT[level] < LEVEL_WEIGHT[this.level]) return noopHot;

    const windowMs = options.windowMs ?? 1000;
    const maxPerWindow = options.maxPerWindow ?? 5;
    const fullTag = this.prefix ? `${this.prefix}:${tag}` : tag;
    const consoleMethod = level === "debug" ? console.log : console[level];

    return (...args: HotArg[]) => {
      const now = performance.now();
      let state = this.hotState.get(fullTag);

      if (!state || now - state.windowStart > windowMs) {
        if (state && state.suppressed > 0) {
          console.log(
            `%c[${fullTag}] throttled: ${state.suppressed} more log entries skipped in the last ${windowMs}ms`,
            "color:#999;font-style:italic",
          );
        }
        state = { count: 0, windowStart: now, suppressed: 0 };
        this.hotState.set(fullTag, state);
      }

      state.count++;
      if (state.count > maxPerWindow) {
        // Over the limit: drop outright without evaluating arguments, avoiding wasted computation
        state.suppressed++;
        return;
      }

      // Lazy evaluation: only these messages that are actually emitted run the function arguments
      const resolved = args.map((a) =>
        typeof a === "function" ? (a as () => unknown)() : a,
      );
      consoleMethod.call(
        console,
        `%c[${fullTag}]`,
        LEVEL_STYLE[level],
        ...resolved,
      );
    };
  }

  /** Create a child logger with a sub-prefix for distinguishing modules */
  child(prefix: string): Logger {
    return new Logger({
      level: this.level,
      prefix: this.prefix ? `${this.prefix}:${prefix}` : prefix,
    });
  }
}

export { Logger };
export const logger = new Logger();

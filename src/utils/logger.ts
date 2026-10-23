/**
 * Minimal logging library
 *
 * logger.debug/info/warn/error — exact line numbers
 * Binds the native console methods directly, so the browser console points at the
 * real business-code call line and can be clicked to jump there. In production
 * (import.meta.env.DEV === false) they become no-ops, eliminated as dead code at
 * build time: negligible size cost, no runtime overhead.
 * Suitable for: ordinary business logs, not called frequently.
 *
 * Usage:
 *   import { logger } from './logger';
 *
 *   logger.info('user logged in', { userId: 123 });
 */

export type LogLevel = "debug" | "info" | "warn" | "error";

export interface LoggerOptions {
  /** Minimum output level, defaults to debug (i.e. everything is emitted) */
  level?: LogLevel;
  /** Log prefix used to distinguish modules, e.g. 'auth' / 'api' */
  prefix?: string;
}

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

class Logger {
  /** Exact-line-number variant: decided at construction whether it really binds to console or to a no-op */
  readonly debug: LogFn;
  readonly info: LogFn;
  readonly warn: LogFn;
  readonly error: LogFn;

  private level: LogLevel;
  private prefix: string;

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
